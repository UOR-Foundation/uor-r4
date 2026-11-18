# REPORT: one-model-end-to-end, codex-50 (the hinge is not the progress measure; the picks are)

Lab: codex. What the 0.43-epoch divergence, the learning-rate A/B and the margin test settle.

## The readings

| arm | step 4,000 top-1 / nearest | step 8,000 top-1 / nearest | dev token hinge (8k) |
|---|---|---|---|
| lr 0.001, margin 1.0 | 312 / 5,666 | **120 / 3,616** | 0.97930 |
| lr 0.0002, margin 1.0 | 482 / 5,472 | **658 / 6,734** | 0.99949 |
| lr 0.001, margin 0.1 | 467 / 5,024 | **75 / 3,660** | 1.00357 (training hinge 0.102) |

DEV is 1,052 windows / 101,175 rows of held-out shards 3–4; step 0 is 16 / 6.

## What it settles

1. **`--lr 0.001` degrades this shape after ~4,000 steps.** Top-1 falls 312 → 120 and `nearest`
   5,666 → 3,616, and on the long run the dev hinge rose above its step-0 value by 47,000 steps.
   **`--lr 0.0002` is the rate for width 512 / 12 layers.** Top-1 482 → 658 and `nearest` 5,472 →
   6,734 (6.7 % of rows) at 8,000 steps, still climbing.
2. **The token hinge is not a usable progress measure at this score scale**, and the margin test
   proves why: the score is `−mean|q − v|` over the carrier's width, so a margin of 1.0 asks for a
   separation the model cannot express. With `--token-margin 0.1` the **training** hinge sits at
   0.102 ≈ the margin, i.e. the target and its nearest rival are still tied, while the model's own
   `Objective::MarginHinge` (margin 1.0, which DEV reports) stays at 0.999. The two are different
   measures and are not comparable across margins.
   **Judge progress by `top-1` and `nearest`, not by the hinge.**
3. **The margin is not the lever either**: lowering it to 0.1 changed the loss scale and made the
   picks worse (75 at 8,000), so the hinge's magnitude was never what held the model back.

## What to run

The 0.85-epoch run at **`--lr 0.0002`**, judged on `top-1`/`nearest` at the four checkpoints, with
the hinge reported as a diagnostic only. `--fast-rivals 1` and `--dev-tokens 200000` stay (2× step,
21× cheaper evaluations).

## What this changes in the registered rule (codex-49)

`R = nearest / top-1` still applies, but the **token hinge is removed from the diagnostic set**: at
0.21 epochs `R` was 14.8 and the hinge had moved 0.01, which is consistent with a working
representation and a hinge at its scale floor — not with a model that cannot learn.

`--token-margin` remains available as a diagnostic; it is not a fix and is not used in a run.
