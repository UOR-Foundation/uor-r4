# Prep sheet: why I-Prefix failed (d11-exporter-float-trunk, claude-140), 2026-10-27

Item [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Order: [claude-140](../../loop/tasks/d11-exporter-float-trunk/claude-140/ORDER.md). Follows kimi-46 (#3306).

## 1. Question

kimi-46 ran the D11 dual-forward training of claude-131 for 2,048 steps. It stopped at its registered I-Prefix gate: its step-1,024 evaluation did not match claude-131's.
- **Step 0 is identical on both seeds.**
- **From step 256 on, the runs differ.** Seed 1's float top-1 at step 1,024 is 3,845 in claude-131 and 1,472 in kimi-46.

**Which of three causes is it?**
- **(a) Same-device nondeterminism.** The FBS backward kernel sums gradients with float `atomicAdd` (`spiralcore_fbs.cu:151`).
- **(b) Source drift.** The training path changed between the two run commits, `faecaaac5` and `e77a1f91`.
- **(c) Cross-device or driver drift.** The binaries carry `compute_120` PTX, which the driver compiles at load time.

The step count is not a cause. The loop's learning rate is constant, and its batches are a document-order prefix (`spiralcore_fbs_sheet.rs:461-495`, `hamilton_sheet.rs:1871-1885`). The same holds for any other function of the total step count.

## 2. Arms and readings

One pod and one RTX 5090. The arms run the two runs' **archived binaries, byte for byte**, with no rebuild:
- **B_new:** kimi-46's `spiralcore-fbs-sheet`, sha256 `804101684d2ae4702169de4028f6a4ef8f44ba5a9fe0a532cce719aeb261d030`, from `results/kimi/d11-fbs-longer-dual-2026-10-26/attempt-2/bin/`, built at `e77a1f91`.
- **B_old:** claude-131's, sha256 `a57a752d7dd3b49d210d18ef16c12f1a54bf5238ae8dc674a616e3512efdb9d8`, from `results/claude/d11-fbs-dual-train-2026-10-26/dual-bin/`, built at `faecaaac5`.

Every arm runs the command both runs used, with the default 1,024 steps and seeds 1 and 2:

```
<bin> --mode d11-train-export --train-forward dual --data-root <data> --save-dir <arm>/save --stage4b <d11-gap.json> --out <arm>/train.json
```

| Arm | Binary | Order |
|---|---|---|
| R1 | B_new | first |
| R2 | B_new | second (repeat of R1) |
| R3 | B_old | third |

**Reading.** For each arm, seed (1, 2) and evaluation point (steps 0, 256, 512, 1,024), take from `train.json`:
- in `.dev_float` and in `.dev_int`: `top1_hits`, `top8_hits` and `admitted_target_hits`;
- `dev_loss_float`, `dev_loss_int` and `train_loss`.

Two points **match** when every hit count is equal and every loss is within 1e-6. That is I-Prefix's own test, applied at all four points.

**Comparisons:**
- **C1:** R1 against R2, the same binary twice on the same device.
- **C2:** R1 against R3, the two binaries on the same device.
- **C3:** R3 against claude-131's archived receipt (`5a290fbe…`): the same binary on two pods.
- **C4:** R1 against kimi-46's archived receipt (`b4ecb87d…`) at steps 0 to 1,024: the same binary on two pods.

## 3. Bar and decision table

The outcome is decided in this order:

| Result | Class | What it means and unlocks |
|---|---|---|
| C1 differs at any point | **(a) same-device nondeterminism** | Exact cross-run equality cannot be reached, so I-Prefix as written is an instrument fault. Next unit: a deterministic gradient reduction in the FBS backward, which changes no math, or a gate that compares within one run. C2 to C4 are reported, not interpreted. |
| C1 matches; C2 differs | **(b) source drift** | The two commits train differently on one device. Next unit: localise the change between `faecaaac5` and `e77a1f91` in the training path. The 2,048-step dose is confounded by code, not by length. |
| C1 and C2 match; C3 or C4 differs | **(c) cross-device or driver drift** | The same binary on the same inputs gives a different result on another pod. Next unit: build SASS for `sm_120` ahead of time, or run every compared arm on one pod. |
| All four match | **none on this pod** | This pod reproduces both archives. kimi-46's pod alone differed (device or driver). Next unit: a same-pod I-Prefix design. |

The first differing step is recorded for every comparison. So is the largest loss difference, at every point.

## 4. Instrument checks (each must be green before the bar counts)

- **I-Bin:** both binaries match their pinned sha256 before they run.
- **I-Inputs:** the five inputs match the claude-131 pins in §5.
- **I-Device:** `nvidia-smi` records the GPU and driver, and `nvcc --version` the toolkit.
- **I-Complete:** each arm's `train.json` holds two seeds, each with points 0, 256, 512 and 1,024.
- **I-Step0:** every arm's step-0 point matches both archived receipts' step-0 points. They matched in kimi-46. A step-0 mismatch means the inputs or binary are wrong, and stops the reading.

## 5. Base artifact and data split

- **Training:** `mw-balp/train`, the same windows both runs used.
- **Evaluation:** the 14,287-row `ft/dev` panel.
- **Inputs, from `w2-inputs/2026-10-25/driver-data/` and the Stage 4b receipt.** These are claude-131's pins:

| sha256 | file |
|---|---|
| `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` | `mw-balp/train/tokens.u16` |
| `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` | `mw-balp/train/response_mask.u8` |
| `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` | `ft/dev/tokens.u16` |
| `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` | `ft/dev/response_mask.u8` |
| `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a` | `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/d11-gap.json` |

## 6. Steps and minimum test power

The test is deterministic, with no statistics. A single differing hit count, or a loss difference above 1e-6, decides a comparison. Two seeds and four points give eight chances per comparison to show a difference. R2 repeats R1 exactly, which is what makes C1 the same-device repeatability test. Each arm trains 1,024 steps, about 176 s per seed (kimi-46 measured 351 s per 2,048 steps), plus five evaluation passes.

## 7. Cost and stop rule

- **Cost:** one RTX 5090 on Vast, about 1 hour including the fetch at roughly $1 per hour, so about $1.
- **Stop rule:**
  - any failed step stops the job, and the results so far are sealed;
  - a hard cap of 2 hours;
  - no arm is rerun under this sheet.
- **Results:** sealed under `results/claude/d11-prefix-repeatability-2026-10-27/` with a SHA256SUMS that is read back before `down`.

## 8. Non-author TEST FITNESS review

To be filled by a fresh-context reviewer given only this sheet and its sources.
