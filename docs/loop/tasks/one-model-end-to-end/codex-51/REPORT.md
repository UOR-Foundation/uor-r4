# REPORT: one-model-end-to-end, codex-51 (the bars, and the writer's admission knob)

Lab: codex. Follows codex-50 (the objective and schedule fix) and the 18 Nov blocker synthesis.

## The bars, measured on the same rows

DEV = held-out shards 3–4, 2,131,471 scored targets (101,175 in the capped evaluation panel).

| reading | count | share |
|---|---|---|
| unigram (commonest target, token 13) | 89,487 / 2,131,471 | **4.20 %** |
| **T2 (commonest successor in TRAIN)** | 340,815 / 2,131,471 | **15.99 %** |
| model `nearest` at 0.45 epochs | 15,500 / 101,175 | **15.32 %** |
| model writer pick at 0.45 epochs | 832 / 101,175 | **0.82 %** |

**The trunk is learning human text**: 3.6× the unigram floor and within one point of a bigram table
after 0.45 epochs, still rising at the last checkpoint. Every `cell` run was behind the unigram.

**The writer is the whole remaining gap**: `R = 15,500 / 832 = 18.6`, and it *worsened* as the
representation improved (`nearest` 11,965 → 15,500 while the pick fell 1,224 → 832).

## The writer's admission knob

`--kept-cells 1..=16` (default 8) names how many of the native writer's 256 cells may open before its
final pick. It is threaded through `build_hamilton_model_full` and `build_fbs_model_full`, omitted
from `config.json` at the default, so every existing run is unchanged. The A/B to run next: 8 against
16 at the same recipe, judged on the writer pick and `nearest` on the same DEV.

The project already knows this knob's failure mode: at the sheet's margin the model flattens its
ports until the hinge reads 1.0000 and the writer scores 345 where whole-table nearest scores 7,813
(claude-142). So the A/B must read the *pick*, not the hinge.

## Running

The longer run at the corrected recipe: `nearest`, `--lr 0.001 --lr-end 0`, batch 24 / micro 6,
32,768 steps = 0.9 epochs, four evaluations, ~5.7 h. Bar: `nearest` above T2's 15.99 %.

## The address, built (codex-52)

The writer's cell address now has a second form behind `--writer-address centroids`: each cell is
scored by the carrier's distance to the cell's centroid over all 512 lanes, derived from the served
table (`token-id mod 256`), no new learned parameter. Off by default, so the port path is unchanged
code and every existing run and test is bit-identical. 4 writer tests pass. The float A/B and the
integer parity unit are ordered in `codex-52/ORDER.md`.
