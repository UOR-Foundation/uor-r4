# ORDER: infra-loop-harness, claude-236 (record four architecture gaps that have no item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `7a1cbcffc`.

Setup row: 2 (item-file maintenance; a records PR, as in claude-221, claude-223, claude-225 and claude-230).

**Written by hand.** `loop claim infra-loop-harness --lab-topic gap-items` refused: "scope docs/roadmap/items/*.md overlaps docs/roadmap/items/integer-hamming-read.md of item integer-hamming-read". That order is claude-235, this lab's own open draft (#3467). This order adds four new item files and touches no file in claude-235's scope.

## Goal

Casey (31 Oct 2026, in chat): "ok well harvest what you can and make sure we follow up on the gaps we are still missing in our own architecture (if we aren't already working on them)", and then: "why is the icosian snap 42% of our time? what is slowing it down? we should improve that process".

The pipeline map's "gaps, ranked" (`docs/model/pipeline-map.md`) lists ten. Gaps 1, 2, 4, 5, 7 and 8 have items:
- gap 1 has `point-key-integer`, `one-model-end-to-end` and `integer-nearest-readout`;
- gap 2 has `writer-admission`;
- gap 4 has `training-step-cost`;
- the read's parity has `integer-hamming-read`;
- gap 7 (the text path) is in `one-model-end-to-end`'s order, and `uor-chat --stack` is in `m7-parity-library-cli-browser`.

Four gaps looked to have no item. This order records each that the cancel probe confirms:
1. **`fbs-snap-sort`** (gap 3's open candidate). The icosian snap is 46% of a served token (claude-177's profile). The candidates `fbs-serving-vector-kernel` left open (claude-186) were "named for a follow-up item", and no item was filed.
2. **`integer-long-context`** (gap 6). The session cannot serve past its 192-token window.
3. **`scale-probe`** (gap 9). No run has measured how ability grows with width or data.
4. **A sampling rule** (gap 10): "Greedy decoding only ... A lawful sampling rule does not exist and has no decision."

## Cancel probe (run first)

- No item already covers any of the four (`grep` over `docs/roadmap/items` for the snap's candidates, sliding or long context, scale and ability, and a sampling rule).
- The roadmap sync's dry run plans no issue for any of the four.

If any fails, drop that item from this order and report it.

Run on 31 Oct at the base:
- **The snap:** the open items naming it (`integer-hamming-read`, `integer-nearest-readout`, `point-key-integer`, `turn-memory-integer`, `m7-parity-library-cli-browser`) mention it only in passing. `fbs-serving-vector-kernel` is `done`, and its open candidates were never filed.
- **Long context:** only `turn-memory-integer` names the window. It records training's 192-token blocks and the float decode's slide (its line 43 and 48), and serves neither.
- **Scale:** no item names a width past 512, the writer's width-1,024 cap, or how ability grows.
- **Sampling: covered, so dropped.** A first grep for "sampling rule" found nothing, but the decode rules of `one-model-end-to-end` are a sampling rule:
  - claude-208 built the Flock rank decode, "the 16 nearest tokens drawn with the integer Flock selector's harmonic weights": a seeded xorshift64* draw, weights `1/(i+1)`, no exponential (`flock_rank_draw`, `in_space_long.rs`).
  - claude-211 took the protocol's decision 4: "Flock rank ships", with the end of turn kept (508 / 510 / 510 complete replies against greedy's 433 / 440 / 432).
  - claude-213 then found that "no decode rule tried yet meets the free-generation condition on new phrasings". That work is open inside `one-model-end-to-end`.

  So gap 10 has an owner, and the pipeline map's sentence is out of date. No item is recorded for it.
- **The dry run** plans 30 body updates and one milestone checklist change. That drift is already on main, from earlier merges, and is not this order's. It plans no issue for any of the four.

Result: **go, for three items**. The probe reads the records, so it is an instrument check.

## Files in scope

- `docs/roadmap/items/fbs-snap-sort.md`, `docs/roadmap/items/integer-long-context.md` and `docs/roadmap/items/scale-probe.md` (all new)
- `docs/roadmap/issues.json` and `docs/roadmap/README.md` (written only by `roadmap sync --apply` and `render`)
- `docs/loop/tasks/infra-loop-harness/claude-236/**`

## Done when

- `xtask roadmap check` passes, `render --check` is clean, and the sync's dry run plans the three new issues on top of the drift already on main.
- Every fact in the three items is quoted from a file at the base, or from a sealed REPORT, with its source named.
- The REPORT gives the negatives first, and a fresh review runs on the diff.
- After the merge, `roadmap sync --apply` creates the three issues.

## Stop and ask if

An item needs a decision only the owner can make: that item then records one question for Casey on #2028 and parks, and the others go ahead.
