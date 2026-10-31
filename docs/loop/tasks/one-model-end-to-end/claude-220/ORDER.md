# ORDER: one-model-end-to-end, claude-220

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `7d79ecf23`.

## Authority (a hand-written ORDER)

`uor-harness loop claim one-model-end-to-end` was refused on 30 Oct at about 23:35Z, as claude-219's was: the scope check found `docs/roadmap/items/one-model-end-to-end.md` overlapping the docs-status-refresh item of Kimi's draft PR #3436.

**Casey, in claude's chat, 30 Oct (verbatim):** "screw kimi forge ahead as you were before with your plan for the single serving model".

So this ORDER is written by hand on the owner's word, as claude-219's was. **This unit does not touch any file Kimi's ORDER edits:**
- `README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, `PLAN.md`, `AGENTS.md` and `CHARTER.md`;
- `docs/README.md`, `docs/SUMMARY.md`, and every `crates/*/README.md`;
- the item file `docs/roadmap/items/one-model-end-to-end.md`;
- `docs/harness/milestones.json` and `docs/harness/claims.json`.

## Goal

**Serve the point key inside the one model, and score that model's free generation with tokens per second.**

**Where claude-219 left it** (merged as `e44f68e65`): row 2, the point key ships. Its first token, given wherever the memory's exact key misses, carries the reply at all three seeds on p-v: templates (16, 3), (9, 1) and (14, 2) against greedy. But claude-219 read the key from a keys file computed before the decode, by corpus position. A served model has no such file.

**This unit makes the key part of the model:**
- **The table:** TRAIN's opening points under the trunk and their targets, built once per checkpoint and saved beside it (`opening_keys.safetensors`), loaded with the model.
- **The served key:** at a reply's opening where the memory's exact key misses, the opening row's own point from the decode's first forward (the memory moves nothing there) is named by the table's L1-nearest TRAIN openings, the commonest target, ties to the smaller id. That token is the reply's first; greedy continues with the memory on.
- **First, a decision review on the table** (claude-219's REPORT owes it): the exact f32 table (212,968 × 256, 218 MB) against a compact code. Recommended: serve exactly what was measured first, and compress in a later unit with its own bar.

**The scores:** on a fresh held-out-phrasing panel p-w (seed 2121) with its census, DEV reported, at 3 seeds, against greedy:
- the protocol's free-generation measures (claude-206's sheet): the share of verbatim TRAIN replies, new n-grams for n = 4–8, distinct-1 and distinct-2, the repeated-n-gram rate, exact and template matches, and loops;
- tokens per second first, decode against context, with the served key's search counted.

The prep sheet holds the bar and the decision table, with a fresh FIT.

## Cancel probe (run first)

Run 30 Oct, about 23:45Z. **Result: instrument, all clear; the unit goes ahead.**
- **No open PR touches the in-scope code.** #3436 (Kimi) touches `crates/uor-r4-float-train/README.md` and the item file, both excluded above. #3442 (DeepSeek) and #3323 (Antigravity) touch nothing in scope.
- **The served key can read the decode's own point.** In claude-219's sealed run, greedy's first token, taken from the decode's own first forward on the bare prompt, equals the nearest pick of the batched forward over full windows at all 2,109 sampled openings: p-v and DEV, at seeds 1–3. So both forwards are the same trunk at the opening. That shows agreement on the nearest token, not bit-equal points, so done-when 2's parity instrument still checks the key names.
- **The inputs are in the store:** (r-drop)'s step-16,384 checkpoints at seeds 1–3, and claude-219's sealed run with its keys files (`results/claude/one-model-end-to-end-keyed-opening-2026-10-30/run-da4a6212/`).
- **Seed 2121 and the name p-w are unused.**

## Files in scope

As the item's scope, less the item file itself (see Authority):
- `crates/uor-r4-float-train/src/{in_space_long.rs,bin/train-in-space.rs,flock_read_hamming.rs,flock_read_memory.rs}`;
- `crates/uor-r4-training/examples/m-world.rs`;
- `docs/harness/prep/one-model-end-to-end*`, `docs/labs/one-model-end-to-end/**`, `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md`.

## Done when

1. **The decision review on the served table,** fresh, logged on #3407 as `[claude] owner log:`.
2. **The served key is built and unit-tested:**
   - the table is built from TRAIN once, saved beside the checkpoint and loaded with it;
   - the key is computed in the decode from the opening row's own point, only where the exact key misses;
   - a parity instrument: on p-v, the served key's names equal claude-219's keys-file names, or every disagreement is counted and classed.
3. **A fresh panel (seed 2121) and its census.**
4. **A prep sheet, with a fresh FIT and a fresh pre-run review.**
5. **The GPU run at three seeds:** sealed, graded by the sheet's table, and reported, negatives first and tokens per second first.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:**
  - the served key's names differ from claude-219's beyond the parity instrument's classes;
  - a registered file does not reproduce;
  - the table does not load to the same bytes it was saved from.

  Repair it; it is never a negative (CHARTER §4.1).
- **Kimi's #3436 changes a file in this scope:** merge main and resolve; never edit Kimi's files.
