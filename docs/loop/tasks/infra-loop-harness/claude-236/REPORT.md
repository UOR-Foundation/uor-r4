# REPORT: infra-loop-harness, claude-236 (record the architecture gaps that have no item)

Lab: claude. ORDER: [ORDER.md](ORDER.md). Base: `7a1cbcffc`. A records PR: no code, no run.

## Negatives first

- **One of the four gaps was already covered, and my first probe missed it.** The pipeline map's gap 10 says "A lawful sampling rule does not exist and has no decision". A grep for "sampling rule" found no item, and I wrote `lawful-sampling-rule`. Reading `one-model-end-to-end`'s reports before committing it found the rule and the decision:
  - claude-208 built the Flock rank decode, "the 16 nearest tokens drawn with the integer Flock selector's harmonic weights": a seeded xorshift64* draw with weights `1/(i+1)` and no exponential (`flock_rank_draw`).
  - claude-211 took decision 4: "Flock rank ships", with the end of turn kept.
  - claude-213: "No decode rule tried yet meets the free-generation condition on new phrasings." That work is open inside `one-model-end-to-end`.

  So that item is not recorded. The miss was the probe's (an instrument fault): a grep on one phrase is not a check that no item covers a gap.
- **Gap 8, "The two model paths do not join", still has no item, and this order records none.** M1 closed on the discrete path (`lut-discrete-trunk/1`). The served replies, and every unit since, are the FBS trunk's (`lut-stack/5`). Whether the discrete path is joined to the trunk, kept beside it or retired is a direction call for `PLAN.md`, not a unit of work, so it is left to the owner's plan. The ORDER's goal counted gap 8 among the gaps with items, which was wrong; the coverage table below is the corrected account.
- **The pipeline map is out of date in two places.** Gap 10 says no sampling rule exists, and gap 6 says "the integer runtime has no look-back", though the Hamming read's copy and type heads are served (claude-194, claude-195). This order does not edit the map. Each new item's done-when updates its own gap.
- **The roadmap sync's dry run plans 30 body updates and one milestone checklist change that are not this PR's.** That drift is already on main, from earlier merges. `roadmap sync --apply` after this merge applies it too, from main's item files.
- **The snap item's two main candidates are reasoning, not code.** The rearrangement argument for golden roots and the 2^-36 bound for the exact sign are written as candidates. The item's done-when tests them against the exact oracles, and says to prove or replace each.

## What changed

Three new items, each with its facts quoted from a file at the base:
1. **[`fbs-snap-sort`](../../../../roadmap/items/fbs-snap-sort.md)** (serving, laptop). The icosian snap is 46% of a served token: 333 of 725 samples, claude-177's profile on claude-126's export. Its share rose 25% → 32% → 42% → 46% as the kernels around it got faster.
   - **Where the time likely goes:** 17 exact comparisons in series, each branching on the data, plus bounds from the multiply ban.
   - **Candidates:** sort the magnitudes and keep at most three golden roots; one fixed-point product for the exact sign.
   - **Bar:** exact against both oracles and an exhaustive 83,521-query sweep, replies byte-identical, the serving audit, and an alternating `fbs-toks --parts` A/B.
   - This is the follow-up `fbs-serving-vector-kernel` (claude-186) named and never filed. Casey asked for it on 31 Oct: "we should improve that process".
2. **[`integer-long-context`](../../../../roadmap/items/integer-long-context.md)** (serving, GPU). The session returns `ContextFull` at 192 tokens. Training cuts 192-token blocks from `<|bos|>`, and the float greedy decode slides over the last 192: three rules past 192.
   - **Candidates:** slide, blocks, or carry the recurrence's state.
   - **Done when:** a measured decision, then a served rule with a parity test, and tokens per second out to 2,048 tokens of context.
3. **[`scale-probe`](../../../../roadmap/items/scale-probe.md)** (training, GPU). The only width points are 256 (`9,328 / 10,573`, claude-152) and 512 (`10,152 / 11,140`, claude-153). The writer caps width at 1,024, and the corpus is 7.5M tokens.
   - It runs widths 256, 512 and 1,024 at several corpus sizes, at least two seeds each, on `one-model-end-to-end`'s recipe.
   - It scores on a fresh held-out panel (P-dev stays reserved) and reports the slope.

`docs/roadmap/README.md` is re-rendered.

## Gates

- `xtask roadmap check`: 121 items OK.
- `xtask roadmap render --check`: current.
- The roadmap sync's dry run: 3 issues to create (exactly the three items), 30 to update and 1 milestone checklist, which is the drift already on main.

## After the merge

`roadmap sync --apply` from a fresh worktree on main creates the three issues.

## Coverage of the ten gaps, for the record

| Gap | Owner |
|---|---|
| 1. Reply opening on new phrasing | `point-key-integer`, `one-model-end-to-end`, `integer-nearest-readout` |
| 2. The served decoder | `writer-admission` |
| 3. Serving speed | `fbs-snap-sort` (new) |
| 4. Training cost | `training-step-cost` |
| 5. The export gap | done at DEV scope; the read's parity is `integer-hamming-read` |
| 6. Long context | `integer-long-context` (new) |
| 7. The text path | `one-model-end-to-end`'s order; `uor-chat --stack` in `m7-parity-library-cli-browser` |
| 8. The two model paths | **no item** (see below) |
| 9. Scale | `scale-probe` (new) |
| 10. Decoding | `one-model-end-to-end` (claude-208, claude-211, claude-213) |
