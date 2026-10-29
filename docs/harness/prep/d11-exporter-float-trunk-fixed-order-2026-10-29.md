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
| O1 | B_base: `spiralcore-fbs-sheet` built on the pod from the same source with `fix.patch` reversed (the three fix files as at `0b441db58`, so float `atomicAdd`), its sha256 recorded | first |
| N1 | B_fixed: `spiralcore-fbs-sheet` built on the pod at the pushed head, before any revert, from the untouched source; its commit and sha256 recorded | second |
| O2 | B_base | third |
| N2 | B_fixed | fourth |

- **The two binaries differ only in the backward:** the three fix files. The FIT found that kimi-46's archived binary differs by nine more training-path commits, so it is not used.
- **Interleaving the arms** keeps any drift over time from favouring either binary.

**Before the arms**, on the same pod:
- **The kernel test:** `cargo test --release -p uor-r4-float-train --features cuda --lib -- --ignored spiralcore_fbs_arms`. It includes `the_backward_gives_the_same_bits_on_every_run` and the existing arm tests.
- **The kernel test's negative gate:** with the fix's lines set aside, the new test runs again on the old `atomicAdd` code. The fix's lines are the two `.cu` files and `geometric_stack_native_cuda.rs`, set aside by `git diff 0b441db58 -- <those three>`, then `git apply -R` and rebuild. Small launches can have a stable atomic order, so this records whether the test can see the old bug at this size. If it passes on the old code, the test is a regression bar only, and the training arms below carry the determinism claim.

**Readings**, for each arm and seed:
- the sha256 of every saved model file;
- from `train.json`, at steps 0, 256, 512 and 1,024: the losses and the float and D11 top-1, top-8 and admitted counts.

## 3. Bar and decision table

| Result | Class | What it means and unlocks |
|---|---|---|
| N1 = N2 in every model file's sha256 and every reading, and O1 ≠ O2 (any file sha256 or reading differs) | **repeatable** | The binaries differ only in the backward, so its `atomicAdd` sums were the cause, and training is now deterministic on one device. Next unit: the export gap re-measured with repeats (R6's second half). |
| N1 ≠ N2 in any file sha256 or reading | **still nondeterministic** | Another order-dependent source remains: candle's own CUDA kernels, or other atomics in our modules. Next unit: dump per-step weight hashes to find the first step and tensor that differ. This is a configuration negative for R6, not a mechanism result. |
| N1 = N2 and O1 = O2 | **no power here** | The old nondeterminism did not show on this pod, so this run cannot tell whether the fix helped. The kernel test still counts; the training reading is inconclusive. Next unit: more steps or a larger batch. |

## 4. Instrument checks (each must be green before the bar counts)

- **I-Bin:** both binaries' sha256s are recorded with their source (the pushed head, and the same with `fix.patch` reversed) before they run.
- **I-Inputs:** the five inputs match claude-140's §5 pins: `tokens.u16` `3dee858f…`, `response_mask.u8` `b5e7352d…`, `ft/dev` `0919e90f…` / `a838bb1a…`, and `d11-gap.json` `88b6a5f9…`.
- **I-Device:** `nvidia-smi`, `nvcc --version`, and `env | grep -E '^(UOR_|CUDA_)'` are recorded before each arm.
- **I-Complete:** each `train.json` has two seeds with points 0, 256, 512 and 1,024.
- **I-Step0:** step 0 matches across all four arms, which share the forward and the inputs. A mismatch means wrong inputs or binary, and stops the reading. claude-140's receipts are compared for information only, since the host code has moved since.
- **I-Kernel:** the kernel test passes; if it fails, stop before the arms.
  - **What it covers:** the new same-bits test and the arm tests that relate the arms to the base path (two paths are a scaled sum, input adjoints are the transpose of their forwards, the frozen arm keeps the exact input adjoint).
  - **What it does not cover:** a GPU-against-CPU parity of the base backward. Its raw output passes through the secant transform before autograd sees it. So a base backward that is deterministic but wrong would still read repeatable. The code review mapped every scratch slot to the old `atomicAdd` target; N's readings are also read beside O's.
- **The negative gate's reading** counts only an assertion failure in `the_backward_gives_the_same_bits_on_every_run`. A build or load error does not count.

## 5. Cost and stop rule

- **Cost:** one RTX 5090 on Vast, about 7 minutes per arm (claude-140), plus the build and fetch. That is about 1 hour at about $0.6/h: **about $1**.
- **Scratch memory:** `rows × outputs × (4·width + 8·lanes)` bytes per projection backward. At the sheet's shapes (batch 4 × context 192 = 768 rows; at most 96 output blocks; width ≤ 64; lanes 8, or 16 with width 128 for the two-path arm) that is at most about 28 MB per call, so no tiling is needed. The new binary's timing is read from `train.json` beside B_base's.
- **Stop rule:** one pod, at most 2 hours. Save everything off the pod before `down`, the saved model files included, to `results/claude/d11-fbs-fixed-order-2026-10-29/`, with `SHA256SUMS` read back. If I-Kernel fails, stop the pod.

Sections 6 to 8 were added after the run, when CI's sheet-shape rule asked for these blocks by name. They restate what §2 and §4 registered before the run, and record the FIT and the pre-run review. They make no new decision.

## 6. Base artifact and data split

- **Base artifact:** none is loaded. Each arm trains from its seed's initialisation, as claude-131, kimi-46 and claude-140 did. The Stage 4b receipt `d11-gap.json` (`88b6a5f9…`) supplies the export settings.
- **Data split:**
  - training on `mw-balp/train` (`tokens.u16` `3dee858f…`, `response_mask.u8` `b5e7352d…`);
  - evaluation on the 14,287-row `ft/dev` panel (`0919e90f…`, `a838bb1a…`), all from `w2-inputs/2026-10-25/driver-data/`.
  The pins are claude-140's §5.

## 7. Steps and minimum test power

- **Steps:** 1,024 dual-forward training steps per seed, on seeds 1 and 2, evaluated at steps 0, 256, 512 and 1,024.
- **Minimum test power:** the test is deterministic. One differing byte in a saved file, or one differing count or loss, decides a comparison.
  - The two earlier runs claude-140 compared (claude-131's and kimi-46's) differed from step 256 on (its §1), so the old nondeterminism shows well within 1,024 steps.
  - O1 against O2 checks that the same holds on this pod.
  - Two seeds and four points give eight chances per comparison.

## 8. Non-author TEST FITNESS review

**FIT (at `18c4d1b39`), fresh non-author reviewer, posted on #2820.** Its findings are folded into the sheet above:
- kimi-46's archived binary differs from B_fixed by nine more training-path commits, so the control is B_base, built from the same source with only the fix reversed;
- a row's "differs" means any file sha256 or reading;
- step 0 is compared within the run;
- I-Kernel names what it covers and what it does not;
- the negative gate counts only an assertion failure;
- the two-path memory estimate;
- the model files are sealed.

**D67 pre-run review (at `470b898ed`): GO**, posted on #2820.

TEST FITNESS: FIT
