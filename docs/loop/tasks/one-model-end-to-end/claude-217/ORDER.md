# ORDER: one-model-end-to-end, claude-217 (step off a reply's own cycle)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `bf0755693`.

## Goal

**Casey's direction** (30 Oct 2026, in the claude lab's chat; recorded verbatim on #3407 6523338624):

> Yes: after claude-216's grade, register a decode unit that steps off a reply's own cycle (the nearest token outside a repeated 4-gram), on a fresh panel, against greedy.

**Why:**
- **claude-213:** 96–99.6% of loop steps admit a single candidate, so rules that spread choice by margin or rank cannot break a loop.
- **claude-216:** training the cut in reduced loops, but cost content (merged as `bf0755693`, "reduces, content lost").

A rule that sees the reply's own cycle acts exactly where greedy loops.

**This unit:**
- **The rule, `DecodeRule::CycleStep`:** at each step, take the nearest table token, unless appending it would close a 4-gram the reply has already produced; then take the nearest of the `k` candidates that does not. If every candidate would close one, take the nearest, and count the step. It is exact and deterministic, with no squashing function.
- **The model:** the sealed (r-drop) at seeds 1–3, the recipe claude-216 kept. There is no training.
- **The arms**, each greedy-based with the memory on, against greedy on a fresh held-out-phrasing panel:
  - **cycle-step:** decides, by Casey's direction;
  - **the decode-only cut:** (r-drop) with claude-216's mask cut, a second registered decision that confirms claude-216's reported "improves" on new prompts;
  - **cycle-step under the cut:** reported.

## Cancel probe (run first)

The result, run 30 Oct 14:30Z and classed **instrument**, gives no cancel:
- **No open PR touches the files in scope.** This unit's own draft, #3428, holds only this ORDER.
- **(r-drop)'s inputs are in the store.** Its sealed step-16,384 checkpoints are at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/r-drop-s{S}/models/step-016384/` (5 files each), and claude-211's registered files are beside them. claude-216's pods fetched and checked all of them against their sums.
- **The panel's seed, 2091, is unused.** The panel seeds in `docs/harness/prep` and `docs/labs` so far are 2001, 2026, 2051, 2052, 2061, 2071 and 2081.

## Files in scope

As the item's scope:
- `crates/uor-r4-float-train/src/{in_space_long.rs,bin/train-in-space.rs,flock_read_hamming.rs,flock_read_memory.rs}`;
- `crates/uor-r4-training/examples/m-world.rs`;
- `docs/harness/prep/one-model-end-to-end*`, `docs/labs/one-model-end-to-end/**`, `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md` and `docs/roadmap/items/one-model-end-to-end.md`.

## Done when

1. **`DecodeRule::CycleStep { k }` exists and is unit-tested:**
   - at `k = 1`, it decodes as greedy;
   - on a hand-built reply, it refuses exactly the token that closes a repeated 4-gram;
   - when every candidate is refused, it takes the nearest.
   - Each sample records which steps stepped off and which were exhausted.
   - `--score-rows` decodes it with `--cycle-step 1`, alone and under `--reply-read all`.
2. **The grade (`--cycle-grade 1`) applies the sheet's registered decisions** with claude-216's bars: loops beyond decision 4's band, holds, close, matched content with its floor, and the loop floor.
3. **A fresh held-out panel and its census.**
4. **A prep sheet, with a fresh FIT and a fresh pre-run review.**
5. **The GPU decode at three seeds:** sealed, graded and reported, negatives first and tokens per second first. The registered files must reproduce byte for byte.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:** a registered file does not reproduce, or the rule leaves greedy before its first refused step. Repair it; it is never a negative (CHARTER §4.1).
- **Too few loops to decide** (the base loops in fewer than 40 replies): the outcome is "underpowered", an instrument outcome.
