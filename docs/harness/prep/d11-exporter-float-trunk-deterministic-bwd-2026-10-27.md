# Prep sheet: a deterministic FBS backward, repeated bit for bit (d11-exporter-float-trunk, claude-141), 2026-10-27

Item [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Order: [claude-141](../../loop/tasks/d11-exporter-float-trunk/claude-141/ORDER.md). Follows claude-140 (#3315).

## 1. Question

claude-140 showed that the D11 dual-forward training is not repeatable on one device. One binary, run twice on one RTX 5090 with the same inputs, trained to different models. Seed 1's step-1,024 float top-1 was 1,780 in one run and 2,914 in the other, and the saved model bytes differed.

The prime suspect is the float `atomicAdd` gradient sums of the pinned FBS backward (`spiralcore_fbs.cu:151`, `:178`). claude-141 adds `src/spiralcore_fbs_det.cu`, the same adjoint with fixed-order gathers, behind `--deterministic-bwd`.

**With it, are two runs of one binary bit-identical?** If not, what still differs?

## 2. Arms and readings

**One binary**, `spiralcore-fbs-sheet`, built on the pod at this branch's pushed head with nvcc 12.8, `UOR_NVCC_ARCH=compute_120`, `--release --locked --features cuda`. Its sha256 is recorded, and the binary is copied off the pod and kept.

| Step | What runs | Reading |
|---|---|---|
| (a) GPU tests | `cargo test --release --locked -p uor-r4-float-train --features cuda --lib`, then the same with `-- --include-ignored deterministic_backward` | All pass, including `the_deterministic_backward_matches_the_pinned_one_and_repeats_bit_for_bit` and `the_deterministic_backward_keeps_the_pinned_error_codes` |
| (a′) negative gate | A copy of the source on the pod with the three dispatch lines in `spiralcore_fbs_bwd` removed (`geometric_stack_native_cuda.rs`, `if …is_deterministic_bwd() { return spiralcore_fbs_bwd_det(…) }`), built in its own target folder, then the same two tests | Both **fail**: the marker `deterministic_bwd_calls()` does not move. A pass means the tests pin nothing, and that is an instrument fault |
| (b) throughput probe | `--mode d11-train-probe --train-forward dual --deterministic-bwd` | `seconds_per_step`, and `projected_full_seconds` (both seeds), which must be ≤ 2,400, that is 20 minutes per seed |
| D1 | `--mode d11-train-export --train-forward dual --deterministic-bwd`, with claude-140's inputs, 1,024 steps, seeds 1 and 2 | `train.json`, and the sha256 of every saved file |
| D2 | the same command again (repeat of D1) | the same |

**Comparison C1-det: D1 against D2.**
- At every evaluation point (steps 0, 256, 512 and 1,024) and on both seeds:
  - all six hit counts are equal;
  - `dev_loss_float`, `dev_loss_int` and `train_loss` are equal **exactly**, not merely within 1e-6.
- The sha256 of every saved `model.safetensors`, `snapshot.json` and `preds-float.json` is equal.

## 3. Bar and decision table

The rows are read in this order, so an instrument fault is decided before C1-det is read.

| Result | Class | What it means and unlocks |
|---|---|---|
| (a) fails; or (a′) passes, or fewer than both of its tests fail; or any §4 check fails | **Instrument fault** | The kernel, its wrapper, its tests or the run is wrong. D1 and D2 are not read, and the instrument is repaired. |
| (b) projects more than 20 minutes per seed | **Stop before D1** | The kernel is too slow as written. It is repaired for speed, and no run is made. |
| (a) passes; C1-det is bit-identical in every reading and every file | **Repeatable** | The float atomics were the whole same-device source. Next unit: I-Prefix's exact gate becomes reachable, within one pod and one binary. The export-gap run kimi-46 set out to do can then be redone under `--deterministic-bwd`. Making the switch the default is a separate D68 decision. |
| (a) passes; C1-det differs | **Another source remains** | Something besides the FBS atomics is nondeterministic, for example candle's CUDA kernels (reductions or `index_add`) or the AdamW update. The first differing step and file are recorded. Next unit: localise it, again under its own sheet. |

## 4. Instrument checks (each must be green before the bar counts)

- **I-Inputs:** the five inputs match claude-140's pins (§5).
- **I-Device:** `nvidia-smi`, `nvcc --version`, and the `UOR_`/`CUDA_` environment are recorded before each arm.
- **I-Receipt:** each `train.json`'s `kernels` block reads `"deterministic_bwd": true` and carries `det_kernel_sha256` and `det_ptx_sha256`, and `pins_match_sheet` stays true.
  - The switch being on is not enough. **`deterministic_bwd_calls` must be at least 2,048** (1,024 steps × 2 seeds, at least one backward per step), and equal in D1 and D2. That shows the deterministic backward ran, not only that the switch was set.
- **I-Step0:** D1's and D2's step-0 points are equal to each other, and equal to claude-140's R1 step-0 points.
  - Step 0 comes before any backward, so this checks the inputs and initialisation only.
  - The binary differs from R1's, so a mismatch with R1 is classed as an instrument fault, not read as a result.
- **I-Complete:** each arm has 2 seeds × 4 evaluation points.

## 5. Base artifact and data split

- **Training:** `mw-balp/train`. **Evaluation:** the 14,287-row `ft/dev` panel. This is the same recipe and data as claude-140.
- **Inputs:**

| sha256 | file |
|---|---|
| `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` | `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16` |
| `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` | `.../mw-balp/train/response_mask.u8` |
| `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` | `.../ft/dev/tokens.u16` |
| `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` | `.../ft/dev/response_mask.u8` |
| `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a` | `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/d11-gap.json` |

## 6. Steps and minimum test power

The test is deterministic. A single differing hit, loss bit or file byte fails C1-det. Two seeds, four points and three file kinds give many independent chances to show a difference. claude-140 found differences at every trained point on both seeds without the switch, so a C1-det match is not a weak-test pass. Each arm is 1,024 steps, about 3 minutes per seed under the pinned kernel. The deterministic kernel's cost is measured by (b) first.

## 7. Cost and stop rule

- **Cost:** one RTX 5090 on Vast for about 1 hour (build, tests, probe and two arms), at about $0.6/h. Under $1.
- **Stop rule:**
  - any failed step stops the job, and what has run is sealed;
  - (b) projecting more than 20 minutes per seed stops before D1. With four seed-runs that bounds training at 80 minutes, inside the cap;
  - a hard cap of 2 hours, held by the pod lease (`--hours 2`);
  - no arm is rerun under this sheet.
- **Results:** sealed under `results/claude/d11-deterministic-bwd-2026-10-27/` with a SHA256SUMS that is read back before `down`. The binary goes there too.

## 8. Non-author TEST FITNESS review

**Round 1 (at `2303fa686`): NOT FIT, with three required fixes, all made.**
1. **I-Receipt showed only that the switch was on.** `kernel_identity` now records `deterministic_bwd_calls`, and §4 requires at least 2,048 calls, equal in D1 and D2.
2. **Failed §4 checks were not routed to the fault row.** That row now covers them and is read first.
3. **The stop rule (b) at 1 hour per seed could exceed the 2-hour cap.** It is now 20 minutes per seed.

Its advisories are kept:
- I-Step0 checks only the forward and initialisation, and a mismatch with R1 is an instrument fault;
- read (a′)'s build line before its result.

The D67 pre-run review passed at the same head, pending this FIT.

**Round 2 (at `fa57c079b`): FIT.** A fresh-context reviewer checked all three fixes in the code and the job script:
- `deterministic_bwd_calls` is counted only in `spiralcore_fbs_bwd_det`, and the train receipt is built after both seeds;
- §3 is read fault-first;
- `projected_full_seconds` covers both seeds.

Its advisory is a thin margin inside the 2-hour lease. Builds, tests and the probe are expected to take about 15 minutes, and D1 and D2 about 15 to 80, so the run stays under the cap.

TEST FITNESS: FIT
