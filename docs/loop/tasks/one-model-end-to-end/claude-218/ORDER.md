# ORDER: one-model-end-to-end, claude-218

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `0ca82f3ff`.

## Goal

**A turn-memory key that a reworded question can reach.** This serves the item's done-when 3 and 4, and Casey's open direction to find the best mechanism that crosses every part.

**Why, from claude-217** (merged as `0ca82f3ff19f24941fea344eedd7477046d8a2d7`), from its sealed rows:
- **Teacher-forced openings, memory on against off:**
  - DEV: 49.1–49.3% against 21.9–23.0%, so the memory adds 322–337 of 1,226;
  - the held-out-phrasing panel p-s: 17.4/13.7/18.6% against 16.4/12.6/17.4%, so it adds 14–16 of 1,400.
- **Later rows** are about 80% on p-s.
- **The memory's key** is the exact token ids of the turn's first `k` slots (`flock_read_memory.rs`).
- **The cycle step** ends every loop, but recovers no content where the reading is powered.

So on new wording the failure is the opening, and the memory that carries the opening on familiar wording does not reach it.

**Casey's direction, 30 Oct (verbatim):** "Yes: finish claude-217 (the cheap decode guard, one pod) first, then build claude-218, the rollout-hinge training fix."
- **This unit departs from its second half,** by the lab's decision review (D68). The review's verdict was GO for the redirect. It is logged on #3407 (6526603691), with a pointer on #2028 (6526608344), and is open to reversal.
- **The rollout hinge is deferred, not negative.** It returns by the decision table below, or by Casey's word.

**This unit, in two steps:**
1. **Step 0, on the laptop (counting over the token files, no model compute).** `--opening-bound 1` gains `--panel NAME=DIR`. For each exact-id key, on the panel's openings and on DEV's, it reports:
   - the fire rate (the key is in TRAIN's table);
   - the precision when it fires (the stored opening is the reference's);
   - the share right overall.

   The keys are the existing first-`k` and last-`k` keys, plus an order-free key (the set of the turn's ids). Beside them sits an oracle ceiling U: TRAIN's commonest opening keyed by the reference reply's own template, with its values masked as the census masks them.
2. **The GPU key probe, under a prep sheet with a fresh FIT and a pre-run review.** It runs on (r-drop) sealed at seeds 1–3, on a fresh held-out-phrasing panel (seed 2101) with its census. p-s is now a development panel: it chooses keys and thresholds, and decides nothing. The arms:
   - the exact key;
   - the trunk's opening point by its Hamming sign cell;
   - the nearest TRAIN opening point under L1, ties to the smaller id;
   - the best id key from step 0;
   - a forced-opening greedy arm: the reference's first token is given, then greedy.

   The probe measures each key's fire rate, precision when it fires, and net gain over the memory-off trunk on the rows where the exact key misses. It splits them by whether the reference is a verbatim TRAIN reply, and reports DEV with backoff (the exact key first). It reports the table's bytes and the search cost per turn.

**The decision table** (the review's), with band `max(2·√(b+c), the spread across seeds)`:
- **A forced correct opening** does not lift exact-reply or verbatim counts beyond the band: the opening is not enough, and the work goes to the hinge or a combined unit.
- **U, minus the larger of the exact-key and memory-off scores,** is inside the band: no prompt key has headroom, and the work goes to the hinge and the later rows.
- **Some key K** has a net gain beyond the band, precision when it fires at least the trunk's, net gain ≥ 0 on the new-question stratum, and DEV not lower beyond the band: the next unit trains and serves the cheapest such K, with an opening bar on another fresh panel.
- **A key has coverage, but its net gain is ≤ 0** because it fires wrongly: the next unit adds an abstention gate.
- **There is headroom, but no K reaches it:** the next unit trains the trunk's opening point to gather paraphrases (a geometric hinge).

## Cancel probe (run first)

The result, run 30 Oct 19:35Z and classed **instrument**, gives no cancel:
- **No open PR touches the files in scope.** This unit's own draft, #3430, holds only this ORDER.
- **(r-drop)'s inputs are in the store.** Its sealed step-16,384 checkpoints are at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/r-drop-s{S}/models/step-016384/` (5 files each, listed at s1 with `uor-store ls`).
- **The panel seed 2101 is unused.** The panel seeds in `docs/harness/prep` and `docs/labs` so far are 2001, 2026, 2051, 2052, 2061, 2071, 2081 and 2091. "2101" appears only as a count in two grade files.
- **Casey has not answered the redirect.** If he chooses the hinge before step 0 lands, this unit stops at its ORDER and claude-218 becomes the hinge.

## Files in scope

As the item's scope:
- `crates/uor-r4-float-train/src/{in_space_long.rs,bin/train-in-space.rs,flock_read_hamming.rs,flock_read_memory.rs}`;
- `crates/uor-r4-training/examples/m-world.rs`;
- `docs/harness/prep/one-model-end-to-end*`, `docs/labs/one-model-end-to-end/**`, `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md` and `docs/roadmap/items/one-model-end-to-end.md`.

## Done when

1. **Step 0 is built and unit-tested, run on the laptop, and its counts are committed** with a reading (`docs/labs/one-model-end-to-end/memory-key-2026-10-30/`). The tests cover:
   - the panel's openings and keys;
   - the order-free key;
   - the oracle U;
   - fire rate against precision on a hand-built table.
2. **The GPU key probe's arms are in `--score-rows`, and unit-tested:**
   - the Hamming-cell key and the L1-nearest key, each built from TRAIN's opening points under the final trunk;
   - the forced-opening arm.
3. **A fresh panel (seed 2101) and its census.**
4. **A prep sheet, with a fresh FIT and a fresh pre-run review.**
5. **The probe at three seeds:** sealed, graded by the decision table above, and reported, negatives first and tokens per second first.

If step 0 already settles the table, by the second row ("no headroom"), the unit closes on step 0 and its REPORT. Steps 2–5 then do not run.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:** the panel's openings do not match its prompts, or a key reads beyond the prompt. Repair it; it is never a negative (CHARTER §4.1).
- **A key needs a dot-product search over all of TRAIN, or any weighting by a squashing function:** refuse it (CHARTER §3.2, §3.3). Use L1 or Hamming distance, and look inside a cell before any full search.
- **Casey chooses the hinge:** stop and switch.
