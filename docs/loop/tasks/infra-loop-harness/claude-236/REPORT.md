# REPORT: infra-loop-harness, claude-236 (record the architecture gaps that have no item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refused on an overlap with this lab's own claude-235 (#3467). Base: `7a1cbcffc`.

This is a records unit: three new items. No code, no run.

## Tokens per second

No measurement. This unit records items. `fbs-snap-sort` records the latest served speed it targets: claude-177's 4,829 / 4,827 decode tok/s on claude-126's export, with the snap at 46% of the step.

## What did not work, first

- **One of the four gaps was already covered, and my first probe missed it.** The pipeline map's gap 10 says "A lawful sampling rule does not exist and has no decision". A grep for "sampling rule" found no item, and I wrote `lawful-sampling-rule`. Reading `one-model-end-to-end`'s reports before committing it found both the rule and the decision:
  - claude-208 built the Flock rank decode, "the 16 nearest tokens drawn with the integer Flock selector's harmonic weights": a seeded xorshift64* draw with weights `1/(i+1)` and no exponential (`flock_rank_draw`).
  - claude-211 took decision 4: "Flock rank ships", with the end of turn kept.
  - claude-213: "No decode rule tried yet meets the free-generation condition on new phrasings." That work is open inside `one-model-end-to-end`.

  So that item is not recorded. The miss was the probe's, an instrument fault: a grep on one phrase does not show that no item covers a gap.
- **Gap 8, "The two model paths do not join", has no item, and this order records none.**
  - M1 closed on the discrete path (`lut-discrete-trunk/1`). The served replies, and every unit since, use the FBS trunk (`lut-stack/5`).
  - Whether the discrete path joins the trunk, stays beside it or is retired is a direction call for `PLAN.md`, not a unit of work.
  - The ORDER's goal counted gap 8 among the gaps with items, which was wrong. The coverage table under "What worked" is the corrected account.
- **The pipeline map is out of date in two places.**
  - Gap 10 says no sampling rule exists.
  - Gap 6 says "the integer runtime has no look-back", but the Hamming read's copy and type heads are served (claude-194, claude-195).

  This order does not edit the map. Each new item's done-when updates its own gap.
- **The roadmap sync's dry run plans 30 body updates and one milestone checklist change that are not this PR's.** That drift is already on main, from earlier merges. `roadmap sync --apply` after this merge applies it too, from main's item files.
- **The snap item's two main candidates are reasoning, not code:** the rearrangement argument for golden roots, and the 2^-36 bound for the exact sign. The item's done-when tests both against the exact oracles, and says to prove or replace each.

## What worked

Three new items, each with its facts quoted from a file at the base:
1. **[`fbs-snap-sort`](../../../../roadmap/items/fbs-snap-sort.md)** (serving, laptop). The icosian snap is 46% of a served token: 333 of 725 samples in claude-177's profile on claude-126's export. Its share rose 25% → 32% → 42% → 46% as the kernels around it got faster.
   - **Where the time likely goes:** 17 exact comparisons in series, each branching on the data, plus bounds left from the multiply ban.
   - **Candidates:** sort the magnitudes and keep at most three golden roots; one fixed-point product for the exact sign.
   - **Bar:** exact against both oracles and an exhaustive 83,521-query sweep, byte-identical replies, the serving audit, and an alternating `fbs-toks --parts` A/B.
   - It is the follow-up that `fbs-serving-vector-kernel` (claude-186) named and never filed. Casey asked for it on 31 Oct: "we should improve that process".
2. **[`integer-long-context`](../../../../roadmap/items/integer-long-context.md)** (serving, GPU). Past 192 tokens there are three different rules: the session returns `ContextFull`, training cuts 192-token blocks from `<|bos|>`, and the float greedy decode slides over the last 192.
   - **Candidates:** slide, blocks, or carry the recurrence's state.
   - **Done when:** a measured decision, a served rule with a parity test, and tokens per second out to 2,048 tokens of context.
3. **[`scale-probe`](../../../../roadmap/items/scale-probe.md)** (training, GPU). The only width points are 256 (`9,328 / 10,573`, claude-152) and 512 (`10,152 / 11,140`, claude-153). The writer caps width at 1,024, and the corpus is 7.5M tokens.
   - It runs widths 256, 512 and 1,024 at several corpus sizes, with at least two seeds each, on `one-model-end-to-end`'s recipe.
   - It scores on a fresh held-out panel (P-dev stays reserved) and reports the slope.

**The ten gaps and their owners:**

| Gap | Owner |
|---|---|
| 1. Reply opening on new phrasing | `point-key-integer`, `one-model-end-to-end`, `integer-nearest-readout` |
| 2. The served decoder | `writer-admission` |
| 3. Serving speed | `fbs-snap-sort` (new) |
| 4. Training cost | `training-step-cost` |
| 5. The export gap | passes at DEV scope; the read's parity is `integer-hamming-read` |
| 6. Long context | `integer-long-context` (new) |
| 7. The text path | `one-model-end-to-end`'s order; `uor-chat --stack` in `m7-parity-library-cli-browser` |
| 8. The two model paths | **no item** (a direction call, above) |
| 9. Scale | `scale-probe` (new) |
| 10. Decoding | `one-model-end-to-end` (claude-208, claude-211, claude-213) |

## What changed

- New: `docs/roadmap/items/fbs-snap-sort.md`, `docs/roadmap/items/integer-long-context.md`, `docs/roadmap/items/scale-probe.md`.
- `docs/roadmap/README.md`, re-rendered.
- This folder's ORDER and REPORT.

## Measurements

None.

## Checks run at the head

- `xtask roadmap check`: 121 items OK.
- `roadmap render --check`: current.
- **The sync's dry run:** 3 issues to create (exactly the three items), 30 to update and 1 milestone checklist (the drift already on main). It is applied after the merge.

## Skills used

None.

## Still open

- **`fbs-snap-sort`** is claimable once claude-235 (#3467) merges, since that order holds `stack/tests.rs` and `m4-serving.md`.
- **`integer-long-context`** and **`scale-probe`** each start with a prep sheet.
- **Gap 8's direction**, if the owner wants the two paths joined.
- **After the merge:** `roadmap sync --apply` from a fresh worktree on main creates the three issues.

## Plain summary

The project's map lists ten things standing between us and a cheap, capable model. Six already had someone working on them, and one (whether the two kinds of model join) is a choice for the plan. This adds the three missing ones:
- make the slowest part of serving faster: the rounding step that takes almost half of each word's time;
- let the served model read past 192 tokens;
- measure whether bigger models get better.

A better way of choosing words than "always the closest" looked missing, but turned out to be already in progress, so it was not added.
