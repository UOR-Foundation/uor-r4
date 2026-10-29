# Prep sheet: does a fixed-order FBS backward make training repeatable? (d11-exporter-float-trunk, claude-179), 2026-10-29

Lab: claude. Item: `d11-exporter-float-trunk` (#2820). ORDER: `docs/loop/tasks/d11-exporter-float-trunk/claude-179/ORDER.md`. R6 in `docs/model/pipeline-map.md`.

## 1. Question

claude-140 found the D11 dual-forward training nondeterministic on one device. One binary, run twice, gave different models: seed 1's step-1,024 float top-1 ranged from 745 to 3,845 of 14,287 rows across five runs. Its prime suspect was the float `atomicAdd` gradient sums in the FBS backward.

claude-179 (head `09040f5be`) replaces them with a fixed-order reduction: scratch, then `spiralcore_fbs_reduce_dx` and `spiralcore_fbs_reduce_dq`. **Question: does the same binary, run twice on one device, now train to byte-identical models?**

## 2. Arms

Every arm runs the command claude-131, kimi-46 and claude-140 used, with the default 1,024 steps and seeds 1 and 2:

`<bin> --mode d11-train-export --train-forward dual --data-root <data> --save-dir <arm>/save --stage4b <d11-gap.json> --out <arm>/train.json`

| Arm | Binary | Order |
|---|---|---|
| O1 | B_old: kimi-46's `spiralcore-fbs-sheet`, sha256 `804101684d2ae4702169de4028f6a4ef8f44ba5a9fe0a532cce719aeb261d030` (claude-140's R1 and R2 binary) | first |
| N1 | B_fixed: `spiralcore-fbs-sheet` built on the pod at `09040f5be` (or the reviewed head), its sha256 recorded before it runs | second |
| O2 | B_old | third |
| N2 | B_fixed | fourth |

Interleaving the arms keeps any drift over time from favouring either binary.

**Before the arms**, on the same pod:
- **The kernel test:** `cargo test --release -p uor-r4-float-train --features cuda --lib -- --ignored spiralcore_fbs_arms`. It includes `the_backward_gives_the_same_bits_on_every_run` and the existing arm tests.
- **The kernel test's negative gate:** with the fix's lines set aside, the new test runs again on the old `atomicAdd` code. The fix's lines are the two `.cu` files and `geometric_stack_native_cuda.rs`, set aside by `git diff 0b441db58 -- <those three>`, then `git apply -R` and rebuild. Small launches can have a stable atomic order, so this records whether the test can see the old bug at this size. If it passes on the old code, the test is a regression bar only, and the training arms below carry the determinism claim.

**Readings**, for each arm and seed:
- the sha256 of every saved model file;
- from `train.json`, at steps 0, 256, 512 and 1,024: the losses and the float and D11 top-1, top-8 and admitted counts.

## 3. Bar and decision table

| Result | Class | What it means and unlocks |
|---|---|---|
| N1 = N2 in every model file's sha256 and every reading, and O1 ≠ O2 | **repeatable** | The atomics were the cause, and training is now deterministic on one device. Next unit: the export gap re-measured with repeats (R6's second half). |
| N1 ≠ N2 | **still nondeterministic** | Another order-dependent source remains: candle's own CUDA kernels, or other atomics in our modules. Next unit: dump per-step weight hashes to find the first step and tensor that differ. This is a configuration negative for R6, not a mechanism result. |
| N1 = N2 and O1 = O2 | **no power here** | The old nondeterminism did not show on this pod, so this run cannot tell whether the fix helped. The kernel test still counts; the training reading is inconclusive. Next unit: more steps or a larger batch. |

## 4. Instrument checks (each must be green before the bar counts)

- **I-Bin:** B_old matches its pin, and B_fixed's sha256 is recorded with its build commit before it runs.
- **I-Inputs:** the five inputs match claude-140's §5 pins: `tokens.u16` `3dee858f…`, `response_mask.u8` `b5e7352d…`, `ft/dev` `0919e90f…` / `a838bb1a…`, and `d11-gap.json` `88b6a5f9…`.
- **I-Device:** `nvidia-smi`, `nvcc --version`, and `env | grep -E '^(UOR_|CUDA_)'` are recorded before each arm.
- **I-Complete:** each `train.json` has two seeds with points 0, 256, 512 and 1,024.
- **I-Step0:** step 0 matches across all four arms and both of claude-140's archived receipts. A mismatch means wrong inputs or binary, and stops the reading.
- **I-Kernel:** the kernel test passes. If it fails, the fix is wrong, so stop before the arms.

## 5. Cost and stop rule

- **Cost:** one RTX 5090 on Vast, about 7 minutes per arm (claude-140), plus the build and fetch. That is about 1 hour at about $0.6/h: **about $1**.
- **Scratch memory:** `rows × outputs × (4·width + 8·lanes)` bytes per projection backward. At the sheet's shapes (batch 4 × context 192 = 768 rows; at most 96 output blocks; width ≤ 64; lanes 8) that is at most about 24 MB per call, so no tiling is needed. The new binary's timing is read from `train.json` beside B_old's.
- **Stop rule:** one pod, at most 2 hours. Save results off the pod before `down`, to `results/claude/d11-fbs-fixed-order-2026-10-29/`, with `SHA256SUMS` read back. If I-Kernel fails, stop the pod.
