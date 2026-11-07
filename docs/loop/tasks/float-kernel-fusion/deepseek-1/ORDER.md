# ORDER: float-kernel-fusion, deepseek-1 (fused quantizer cost and default)

Lab: deepseek. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md)
(#3678). Base: `f66f023d1`.

## Goal

Complete the item's done-when 4 and 5 on the current training line:

> 4. **The cost,** on an RTX 5090 at P2b's recipe: kernel launches and seconds per training
> step, chain against fused, by the trainer clock (and `nsys`, if the pod has it).
> 5. The flag defaults on only when 2 and 3 hold. One notebook row and a REPORT.

Claude-285's done-when 2 and 3 already hold on an RTX 5090. This unit measures the fused
quantizers on the current b64L human-text recipe, checks that 256-step checkpoints remain
bit-identical, and turns fusion on for `train-in-space` only if the measured step improves.

## Claim check

- **The immediate project need:** Claude's D3 token-count run is at 0.633 / 0.698 s a step for
  16,384 steps. The same run at 32,768 steps is projected near 14.6 h for two seeds and roughly
  $8.50. Every repeat of a multi-hour run pays the fused quantizers' current cost again.
- **What changes in the model:** no value or gradient. `activation_quantize16` and
  `q16_residual` already execute their exact fused replacements under `--fused-quantize 1`;
  claude-285 proved the losses, gradients, samples and eight-step checkpoints bit-identical.
  This unit extends the equality window and changes the default only if the measured clock says
  the fused path is faster.
- **Claude's work is isolated:** this branch does not touch `claude/d3-tokens`, Claude's two
  running pods or their files. The source change, if earned, is one trainer default plus its
  docs and tests on `deepseek/float-kernel-fusion`.

## Cancel probe (run first)

1. `--fused-quantize` is present at the base, the fused PTX is built from its own module, and
   claude-285's GPU equality gates passed at `a1588bcc4`.
2. The current b64L command reaches the flag under `--train-as-served 1`, and the option's
   explicit `0` remains available for the chain arm.
3. This branch owns `deepseek/float-kernel-fusion` in `docs/harness/claims.json`, and no other
   claim names the files this unit changes.

If any check is false, stop and re-scope before compute.

## Files in scope

- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/fused_quantize.rs` (test-only if needed)
- `docs/harness/prep/float-kernel-fusion-fused-cost-2026-11-07.md`
- `docs/loop/tasks/float-kernel-fusion/deepseek-1/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/float-kernel-fusion.md`

No kernel source, model objective, data, checkpoint or serving artifact changes.

## Done when

1. The laptop checks pass: the float-train library tests, the focused flag-default test,
   stub-CUDA `cargo check`, `cargo clippy` for the crate, `git diff --check`, the loop report
   and local PR checks.
2. On one leased RTX 5090, four 256-step b64L runs execute in the registered order
   `OFF, ON, ON, OFF`, with equal seeds, data, micro-batch and eval cadence.
3. The step clock is read over steps 128 to 256 in every run, after warm-up, and the ON/OFF
   gain is reported against the OFF and ON pair spreads.
4. All four step-128 and step-256 model files are byte-identical, and the loss columns agree.
   Any difference fails the correctness gate before a speed result is read.
5. The launch count is reported from the measured call counts and the registered tensor-op
   counts for each chain. An `nsys` launch count is added if the pod provides it; otherwise
   that absence is explicit.
6. If the gain is above noise and at least 0.5% of a step, `train-in-space` defaults
   `--fused-quantize` to `1`; an explicit `0` still selects the chain. Otherwise the default
   stays `0`, and the REPORT closes the performance configuration without calling the geometry
   negative.
7. The RTX 5090 run is sealed and read back, the pod is down, and one notebook row and a
   negatives-first REPORT record the result.

## Stop and decide

Ask means decide, record and continue.

- **Checkpoint or loss mismatch:** stop the speed reading. Classify instrument or kernel, keep
  the default off unless the exact cause is repaired and the run repeats.
- **Fewer than three usable 128-step intervals:** add one balanced block on the same pod.
- **Cost or pod cap:** one RTX 5090, at most $3 and 40 minutes. A healthy run is not cancelled
  for a self-set budget; extend and record instead.
- **Other lab's pod:** never lease, reuse or inspect its GPUs. Create our own within the shared
  caps or stop compute and finish the code review.
