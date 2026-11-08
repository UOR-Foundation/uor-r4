# Prep sheet: 2,048-step reply-SFT dose extension (one-model-end-to-end, deepseek-28)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [deepseek-28](../loop/tasks/one-model-end-to-end/deepseek-28/ORDER.md).

TEST FITNESS: FIT - pending non-author review.

PRE-RUN: PENDING (fresh D67 review required).

## Why

The reviewed 1,024-step reply-masked SFT selected step 512 with continuation
142/512, teacher-forced EOS top-1 0/256, top-8 256/256, median rank 2, free EOS
stops 23/256 and zero complete accepted replies. That is outcome row 6, partial
movement. The registered next step is one 2,048-step dose extension at a new
root.

## Question

With the same `(b64L)` seed-1 base, mixed data, reply mask, optimizer and
selector, but 2,048 steps instead of 1,024, does the selected checkpoint clear
an outcome row above row 6 while preserving continuation?

## Fixed base and inputs

Use the exact base, data, tokenizer, held-out guard, panel and hashes in
[the original sheet](./one-model-end-to-end-reply-sft-2026-11-07.md). The
1,024-step run and its seal remain immutable at:

`hf://datasets/caseyallard/uor-r4-store:results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-20261108t005743z-a849820fba9cfbf4`

## Recipe

The original recipe with one change:

```text
--steps 2048
--eval-every 256
--seed 1
```

All other flags, hashes, base checkpoint/config/Hamming files, batch size,
micro-batch retry and scoring commands are unchanged. Training starts from the
same base at step 0 in a new root. It does not continue from the selected
step-512 checkpoint.

## Readings and selector

Record the same base controls and all checkpoint readings. Apply the selector
and `per-turn/v1` ordering from the original sheet unchanged across checkpoints
256 through 2,048.

## Outcome

Apply the original outcome rows 1-5 to the selected checkpoint. Row 2 may stop
the trainer at step 512 exactly as before. Because this run is the one row-6
extension, another row-6 result closes this dose; it does not authorize a second
extension. Row 7 also closes the dose.

- Row 1: continuation failure; no seed 2.
- Row 2: no ranking movement; stop and close this configuration.
- Row 3: positive; unlock seed 2 and a longer run from the selected checkpoint.
- Row 4: termination moved but complete content did not; diagnose first
  divergence, then a different answer-content intervention.
- Row 5: ranking moved but free generation did not; diagnose the mismatch.

No result claims M2, a mechanism effect, seed reproducibility or useful chat.

## Inherited checks

All original input, overlap, initialization, checkpoint-completion, panel,
finite-progress, seal/readback and teardown checks remain mandatory. OOM allows
one micro-batch-8 retry. Instrument failure trains nothing. A second OOM, changed
input, non-finite progress, missing checkpoint, failed export/panel or seal
failure stops the dose. Never cancel a healthy run for a self-set timer.

## Minimum test power and cost

One seed is an exploratory dose reading. Seed 2 remains conditional on row 3 and
a fresh decision. The 1,024 run took 721 seconds of training and 1,457 seconds
of pod uptime. Eight checkpoints and 24 minutes of training project to 45-50
minutes and roughly $0.45-$0.55 at $0.563/hour. One OOM retry projects to under
$1.30. The launcher remains inside four pods and $8/hour.
