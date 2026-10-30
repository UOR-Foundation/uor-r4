# Flock rank inside the nearest token's cell (claude-213, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Order: [claude-213](../../../loop/tasks/one-model-end-to-end/claude-213/ORDER.md). Sheet: [one-model-end-to-end-flock-cell-2026-10-30](../../../harness/prep/one-model-end-to-end-flock-cell-2026-10-30.md), FIT and pre-run PASS. Direction: Casey, 30 Oct, "do not rule flock out, improve it instead".

## The run

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1, 2 and 3.
- **The rules:** greedy, `FLOCK_END`, and `FlockCell` at six margins (1.0, 0.5, 0.25, 0.125, 0.062, 0.031 of a score).
- **The data:** 256 replies per rule, split and seed, on DEV and on the new panel p-c.
- **The hardware:** one RTX 5090 (`vast-58366166`), built at `c9fab4576`. Binary sha256 `57b784b08b6fe276d8d5f4e68a86ee72bce73ae3a3649fd3407c15f003ba6d80`.
- **The checks:**
  - 222 CUDA tests pass on the pod, among them `the_cell_rule_at_margin_0_decodes_as_greedy_on_a_model`;
  - every input matches its store sha256;
  - the binary reproduces claude-211's 15 registered files byte for byte, with the three seeds run side by side.
- **Sealed** at `results/claude/one-model-end-to-end-flock-cell-2026-10-30/run-c9fab457/`. Its 120 files are in a `SHA256SUMS` and were read back OK.
- **Graded** on the laptop by `train-in-space --cell-grade 1` at `c9fab4576`: [cell-grade.json](cell-grade.json), sha256 `3f6b9939ae83c2665ef4d82f344ca4e49f0bb50d4ae315664aac58946affe2d9`.

## Tokens per second

- **Every rule decodes at 58–60 tok/s per process,** with three processes sharing the one GPU, about 180 tok/s together. This is the float GPU path with the memory on, at a context of up to 192.
- **The cell rule costs nothing over `FLOCK_END`:** both read the same top-16 sort.
- **Nothing here is the served speed.** The integer path for (r-drop) is not built yet.

## The registered outcome

| Step | Result |
|---|---|
| DEV's selection | **0.25, "closest, not qualified".** No margin is close at any DEV seed. Of the three that hold decision 4 at all three seeds (1.0, 0.5, 0.25), 0.25 is nearest the references. |
| p-c's outcome | **narrows.** 0.25 holds decision 4 at all three seeds, and its `|v − v_ref|` is below `FLOCK_END`'s at all three. As the sheet registered, this means "holds, not close": the salad is reduced, not cured. |

## p-c, the confirming panel

The references' verbatim share `v_ref` is 0.59 and their repeat rate is 0.00; `tol` is 0.10–0.107. Each cell is the range over the three seeds.

| Rule | Complete, of 256 | `v` | Repeated-4-gram rate | Mean admitted, of 16 | Replies identical to `FLOCK_END`'s |
|---|---|---|---|---|---|
| Greedy | 192–202 | 0.381–0.469 | 0.559–0.628 | — | — |
| `FLOCK_END` | 251–256 | 0.000–0.008 | 0.001–0.002 | 16 | 256 |
| margin 1.0 | 250–256 | 0.016–0.035 | 0.001 | 15.13–15.18 | 195–209 |
| margin 0.5 | 247–256 | 0.051–0.098 | 0.001–0.058 | 13.46–14.43 | 132–167 |
| **margin 0.25** | 229–251 | 0.116–0.184 | 0.076–0.332 | 9.09–12.13 | 59–101 |
| margin 0.125 | 225–237 | 0.232–0.289 | 0.308–0.409 | 6.43–8.29 | 17–51 |
| margin 0.062 | 202–229 | 0.310–0.356 | 0.376–0.565 | 3.70–5.28 | 4–16 |
| margin 0.031 | 190–206 | 0.343–0.353 | 0.553–0.614 | 2.21–3.07 | 0–3 |

On DEV, whose prompts are TRAIN-templated (`v_ref` 0.57), the same shape holds:
- **Greedy:** 227–248 complete, `v` 0.560–0.595, repeat 0.237–0.424.
- **margin 1.0:** `v` 0.047–0.090, repeat 0.001–0.002.
- **margin 0.031:** `v` 0.577–0.616, repeat 0.248–0.424.

## What it shows

- **The objective's own margin admits almost all 16.** The model has not separated its top tokens by the 1.0 it was trained to: within 0.25 of the nearest there are still 9–12 candidates.
- **`v` and the repeat rate rise together along the ladder.** No margin is both TRAIN-like and loop-free, at any seed, on either split. Narrowing the draw moves along one curve from `FLOCK_END` (no loops, no TRAIN-like replies) to greedy (both).
- **The loops are not near-ties.** At 0.031, with 2–3 candidates admitted, the repeat rate is greedy's own (0.55–0.61). The steps where greedy loops are ones where the nearest token is already well ahead, so a narrowing that only breaks ties cannot break them.
- **What a narrowed draw does fix,** in the fixed reading of seed 1's first 20 p-c replies:
  - **Two replies become the reference's kind of reply:** reply 3 is "Okay, I'll remember that." and reply 17 is "That's great news about the bike!". Under `FLOCK_END` they were "Okayist coneBilly" and "That's greatist bark'teded legs2setchund.".
  - **Nine of the 20 are character for character `FLOCK_END`'s salad** (replies 0, 4, 5, 8, 11, 12, 14, 16 and 18).
  - **Two loop** (replies 2 and 6).
  - **Greedy's complete replies are often fluent,** though not always the reference's content. Reply 5 is "I'm sorry you feel nervous. I hope it gets better soon." where the reference says "Got it, you grew up in Hirplerfield.". Its repeat rate comes from the replies it never ends, like reply 4's "7 plus 7 plus 7 …".

## What it unlocks

As the sheet registered for "narrows":
- **The cell rule stays the Flock candidate, and it is iterated.**
- **The next narrowing comes from the same geometry:** the Hamming read's address cell in place of a score margin.
- **The design input for that unit is the loop finding above.** The loops sit where the nearest is confident, so the next rule has to act on those steps too, not only on ties.
- **The score-margin cell at these six margins is a negative for this configuration only:** (r-drop) at step 16,384, top-16, harmonic weights.
