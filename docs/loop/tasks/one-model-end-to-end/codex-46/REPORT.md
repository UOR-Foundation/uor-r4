# REPORT: one-model-end-to-end, codex-46 (the first general-text curve: addresses collapse, tokens do not)

Lab: codex. First run of the geometric trunk on real instruction data, with a donor DEV.

## The curve (30 M model: 512 width, 12 layers, 8,192 vocab, 512 context)

| step | epoch | dev top-1 | nearest | openings | top-8 | admitted | dev loss (token + address) | train |
|---|---|---|---|---|---|---|---|---|
| 0 | 0.00 | 281 | 225 | 0 / 4,341 | 2,216 | 68,116 | 2.83817 (1.00197 + 1.83620) | 0 s |
| **500** | **0.36** | **2,598** | **35,206** | **0 / 4,341** | 9,381 | 66,769 | **2.00441 (1.00232 + 1.00209)** | 371 s |

DEV is 14,869 windows / 2,131,471 scored rows from 4,096 held-out donor conversations.

**What moved:** nearest 225 → 35,206 (**156×**); the **address term 1.83620 → 1.00209**, which is
essentially its floor; admitted positions 68,116 → 66,769 (flat).
**What did not move:** the **token term 1.00197 → 1.00232** (flat, still unsatisfied); **reply
openings 0 / 4,341**.

So at 0.36 epochs the trunk has learned *where* on real chat and not yet *what*. That is the same
signature the narrow world produced from the other side: the address is easy, the token decision is
the work.

## Cost, measured (correcting my own inference)

**371 s for 500 steps = 0.74 s/step** at batch 8 × context 512 with `--micro-batch 2`, i.e.
≈ 5,500 tokens/s. A 332 M-token epoch is therefore **≈ 17 h**, not the ~80 h I inferred from wall
clock between evals — the delay was the DEV evaluation, not the step.

## What did not work

1. **Two CUDA OOMs.** The first at the first training step with `--batch 8` and no micro-batch; the
   second at the **third** DEV evaluation. `--micro-batch 2` fixes the training step; the evaluation
   appears to leak or fragment across repeats, so this run carries **two** evaluations only
   (`--eval-every 2000`). A DEV evaluation at this shape costs ~12 min and 2.13 M scored rows.
2. **My 3.5 s/step inference was wrong** and would have cancelled a viable run; the step time is
   measured, not inferred, from here on.

## What this decides

The token term is the bar. If it is still ~1.0 at 1.42 epochs, the trunk or the writer cannot
separate next tokens at this shape on real text, and the answer is the writer's admission measured
against a count table — not more data. If it falls, the long run is authorized.

The OOM at repeated evaluation and the 12-minute DEV cost must be fixed (subsample DEV, or free the
training tensors before evaluating) before any multi-epoch run.
