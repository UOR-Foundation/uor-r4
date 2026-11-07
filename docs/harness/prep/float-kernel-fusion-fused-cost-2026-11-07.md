# Prep sheet: the fused quantizers on b64L (float-kernel-fusion, deepseek-1, 7 Nov 2026)

Lab: deepseek. Item: [float-kernel-fusion](../../roadmap/items/float-kernel-fusion.md)
(#3678). Order: [deepseek-1](../../loop/tasks/float-kernel-fusion/deepseek-1/ORDER.md).

TEST FITNESS: PENDING (a fresh non-author review must precede compute).

## Why

Claude's current one-model path trains as served at batch 64 in four micro-batches of 16.
Two served rounding chains run throughout that forward:

- `activation_quantize16`, with its straight-through bridge, is about 49 tensor ops a call;
- `q16_residual`, with its bridge, is about 13.

claude-283 measured the chains' removal at 7.51% of a step at 192 x 16 and 2.58% at 2,048 x 4.
claude-285 built one fused forward and backward kernel for each chain and showed the fused path
bit-identical:

- each op against its chain on edge rows;
- one training step's loss and every gradient, at short and long context;
- eight steps' models, samples and progress values.

The true fused gain was never measured, and the training flag still defaults off. Claude now
needs a 32,768-step D3 run at about 0.63-0.70 s a step. This run decides whether future repeats
can receive the smaller launch graph by default.

## Question

At the current (b64L) human-text recipe on one RTX 5090, how much trainer-clock time per step
does the fused quantizer path save against the stock chains, and does 256 steps remain exactly
equal?

## Base artifact

- **Pin:** the reviewed head of `deepseek/float-kernel-fusion`, whose source starts at
  `f66f023d1`. A code change after review gets a new pin and review.
- **Binary:** `train-in-space`, built `--release --locked --features cuda` on the pod, with its
  SHA-256, `nvcc --version`, driver version and GPU UUID recorded.
- **Initial model:** fresh seed-1 initialization in every run. No checkpoint is resumed.

## Data split

| Input | Store path | SHA-256 |
|---|---|---|
| TRAIN D3 | `results/claude/scale-probe-2026-11-05/inputs/d3` | `tokens.u16` `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`; `response_mask.u8` `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875` |
| DEV, in-run checkpoint evaluation | `w2-inputs/2026-10-25/driver-data/ft/dev` | `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| tokenizer | `w2-inputs/2026-10-25/driver-data` | `tokenizer.json` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

No held-out panel is scored. The final DEV evaluation is the trainer's ordinary checkpoint
read and is not a bar.

## Arms

All four runs use the same b64L recipe:

```text
--data-root DATA
--train-split results/claude/scale-probe-2026-11-05/inputs/d3
--objective nearest
--retain-rates 1
--decay-mult 1
--lr 0.001
--lr-end 0
--train-as-served 1
--context 192
--width 256
--hamming-read 4
--batch 64
--micro-batch 16
--steps 256
--eval-every 128
--seed 1
--samples 0
--fused-quantize MODE
```

The sequence is `OFF, ON, ON, OFF`:

| Run | Mode | Purpose |
|---|---|---|
| A0 | `0` | first chain arm |
| B0 | `1` | first fused arm |
| B1 | `1` | fused repeat |
| A1 | `0` | chain repeat |

The balanced order separates a monotonic host or thermal drift from the flag. Every run writes
to a new folder and writes `fused-quantize.json`.

## Readings

- **R1, step rate.** `progress.jsonl`'s `train_seconds` is cumulative training time and excludes
  the checkpoint evaluation between rows. For each run:
  `rate = (train_seconds(256) - train_seconds(128)) / 128`.
  This drops the first 128 steps from the timing reading and retains two checkpoint hashes.
- **R2, gain.** Let `off = (A0 + A1) / 2` and `on = (B0 + B1) / 2`.
  `gain = (off - on) / off`.
- **R3, noise.** The larger of:
  - `|A0 - A1| / off`;
  - `|B0 - B1| / on`;
  - 0.25% of a step.
- **R4, exactness.**
  - `models/step-00128/model.safetensors` and `models/step-00256/model.safetensors` are
    SHA-256-compared across all four folders;
  - the `train_loss_mean` fields at steps 128 and 256 are compared as exact text;
  - both ON runs' `fused-quantize.json` must report nonzero fused counts for activation and
    residual, and both OFF runs must report nonzero chain counts for each.
- **R5, launches.** Counts from `fused-quantize.json` are reported with the source's tensor-op
  count per call. The estimated training launches are
  `2 x (activation_calls x 49 + residual_calls x 13)` for OFF and
  `2 x (activation_calls + residual_calls)` for ON, where the factor 2 counts forward and
  backward. If `nsys` is present, one 32-step smoke is profiled and the actual kernel launch
  count is reported beside the estimate.

## Bar

| Outcome | Reading | What it unlocks |
|---|---|---|
| **Adopt** | R4 holds, `gain > 2 x noise`, and `gain >= 0.5%` | Default fusion on for `train-in-space`; explicit `0` remains the reference. |
| **Keep off** | R4 holds and `gain <= noise` | Keep the default off, record the measured fused cost, and close done-when 4 for this recipe. |
| **Inconclusive** | R4 holds and the gain lies between | Run one more balanced `OFF, ON` block on the same pod if within budget; otherwise keep off and report inconclusive. |
| **Fail** | Any R4 equality or count gate fails | No speed reading and no default change until the instrument or kernel is repaired. |

The bar is about cost, not model quality. No loss, checkpoint or capability claim beyond exact
equality is made.

## Instrument checks

- **I-0:** the crate's CUDA library suite passes with `--test-threads=1`, including
  `one_step_as_served_is_bit_equal_with_the_fused_quantizers` and the fused op tests.
- **I-1:** every run exits 0 and both progress rows exist.
- **I-2:** `config.json` records the b64L settings, seed 1 and the requested mode; the D3 and DEV
  hashes match the table.
- **I-3:** each folder's `fused-quantize.json` records the requested mode and counts both
  paths. An OFF run with any fused call, or an ON run with any chain call on CUDA, fails I-3.
- **I-4:** A0 and A1 are an A/A pair: their losses and checkpoints must agree exactly before
  the chain arm is used as a time baseline.
- **I-5:** `nvidia-smi` records the same GPU UUID in every run, and no other process uses it.

## Steps

1. Receive FIT and a pre-run review of the chain, binary pin, job and stop rule.
2. Run `uor-pod status`. Create and lease our own one-card RTX 5090 pod inside the caps.
3. Bootstrap the reviewed pin, build the release binary and run I-0.
4. Fetch the three inputs, check their SHA-256, and run A0, B0, B1 and A1 in that order.
5. Seal the job folder with `SHA256SUMS`, `DONE` and `EXIT`; put it in the store and read it
   back before taking the pod down.
6. Grade R1-R5 on the laptop. Apply the bar, change the default only if earned, run the laptop
   checks, and write the REPORT and notebook row.

## Minimum test power

Four 128-step readings after warm-up, balanced two per arm. Each is about 80 s of training.
The OFF and ON repeats measure local run drift directly. Fusion is a value-preserving graph
replacement, so no seed or panel replication is needed for a timing decision.

## Cost

- About 12 minutes of four runs at the measured 0.63-0.70 s a step.
- About 5 minutes for bootstrap, build, I-0, hashes and sealing.
- About 5 minutes of contingency, with a hard stop and report at $3 or 40 minutes.

## Stop rule

- On a failed I-0, input hash or run exit, seal evidence and stop; no timing is read.
- On an R4 mismatch, stop the timing read and classify it before another run.
- Never touch Claude's pods. If no own pod can be created under the caps, finish the code and
  document review, leave compute NOT_RUN and do not substitute the laptop CPU.
