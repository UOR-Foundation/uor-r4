# REPORT: integer-nearest-readout, claude-246 (Q1 to Q3 of claude-245's bar, on sealed files): HELD

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Sheet: [integer-nearest-readout-grade-2026-11-01](../../../../harness/prep/integer-nearest-readout-grade-2026-11-01.md). Its FIT was posted on #3462 as 6562777126 and its pre-run review as 6562928526. References #3462.

## Tokens per second

No new decode speed is claimed, and no decode ran. This unit grades sealed decodes:
- claude-244's six forced decodes of R;
- claude-243's greedy replies of R and F′, and F′'s rows.

R's decode speed is still claude-243's: a median of 1,204 tok/s decoding R s1 on this M1.

## What did not work, first

- **Done-when 3 is still not met.** Its Q5 asks for every clear-margin disagreement to be accounted for by an exact trace, and that needs its own decision card. So whatever Q1 to Q3 read, the item stays `ready` (ORDER, done-when 5).
- **R loses slightly in some strata, though every loss is inside its bound.** b − a is R's net loss: the positions only F′ gets right, less those only R gets right. The strata where it is above zero:
  - **DEV under teacher forcing:** +1, +6 and +2 per cell, and +9 of 12,441 pooled. DEV s2's Q2 is one of the tightest margins: a net loss of 6 against a bound of 20.
  - **DEV's pooled opening (Q3):** +2 of 852, so (b − a)² = 4 against 9V = 18.
  - **p-w's cut windows under teacher forcing:** +12 of 4,254, so 144 against 468. This is as tight as DEV s2's Q2, or tighter, depending on the measure.
  - **p-w s2 under teacher forcing:** +2 of 13,931.
  - **p-w s1's opening (Q3):** +1 of 1,067.
  - **Among the readings, which are not tested:** +1 at DEV s1's and s3's openings, and +1 on DEV's pooled cut windows under teacher forcing.
- **The pre-run review failed its first round,** on one Major. No non-author had read the runner's grading section since a FIT fix changed it. FIT round 4 read it, and the pre-run review's round 2 passed (sheet §9 and §10).
- **The FIT took four rounds.**
  - **Round 1** was recorded NOT_FIT: the reviewer reached its context limit before it finished. It found two Minors and three Nits, all fixed, and no Blocker or Major.
  - **Round 2** confirmed those fixes.
  - **Round 3** was split into three small jobs. It covered the sampler path and the state carried between forwards, and the sheet was recorded FIT after it.
  - **Round 4** followed the pre-run review's round 1. It fixed six runner Minors and Nits and confirmed the author's line citations.
  - **No single non-author round gave FIT on the whole sheet.** Rounds 1 and 2 ran out of context, and round 3 was three narrow jobs. The `TEST FITNESS: FIT` line is the author's summary of rounds 1 to 3, and it comes before round 4 changed the runner. What covers this gap is the pre-run review's round 2, a fresh non-author, which read the whole sheet at the final runner hash and passed.
- **Three of the pre-run review's Nits were left in place.** Changing them would move the reviewed runner hash, and none changes an outcome (sheet §10):
  - I-NoKey would also accept an object of rows;
  - a `jq` inside an `echo` can fail without stopping the run;
  - the join-agreement check fires only if an input changes between its two passes.
- **The sheet's LOSS row is stricter than the card.** It adds §8's three conditions to the card's exception for a void reading. That does not touch a HELD reading, but the row is not card text (the exact-head review's Nit).
- **Disclosed before the run** (sheet §5.3):
  - **F′'s own score was seen.** The author saw F′'s own `nearest_hits` in claude-243's GPU log: F′ alone, over every scored row of DEV and p-w at seed 1 (11,841 of 14,287 and 37,049 of 49,251). That gives neither a nor b.
  - **No R total appears in any log.** None of R's logs prints its total against the reference; this was checked with `grep -c`, without showing a value.
  - **claude-244's checks are only upper bounds.** claude-244's `grade.log` printed `forced_greedy_checked` and `align_checked`, which only bound the right openings from above.
- **The disk is at 26 GiB free,** under the 30 GiB floor. This unit built nothing, so I have no build folder of my own to delete. `local/claude/claude-243/` and `claude-244/` stay, because they are Q5's inputs.

## What worked

- **The outcome is HELD.** All 29 tests hold: 23 read here and Q4's 6 carried. A further 5 strata are readings, so the card does not test them. No tested stratum fails.

| Teacher forced (Q1, Q2) | n | a (R only) | b (F′ only) | b − a | V | Q1 | Q2 bound | Q2 |
|---|---|---|---|---|---|---|---|---|
| DEV s1 | 4,147 | 14 | 15 | +1 | 31 | holds | 20 | holds |
| DEV s2 | 4,147 | 6 | 12 | +6 | 32 | holds | 20 | holds |
| DEV s3 | 4,147 | 12 | 14 | +2 | 26 | holds | 20 | holds |
| p-w s1 | 13,931 | 71 | 54 | −17 | 133 | holds | 69 | holds |
| p-w s2 | 13,931 | 73 | 75 | +2 | 154 | holds | 69 | holds |
| p-w s3 | 13,931 | 88 | 81 | −7 | 161 | holds | 69 | holds |
| **DEV pooled** | 12,441 | 32 | 41 | +9 | 79 | holds | 62 | holds |
| **p-w pooled** | 41,793 | 232 | 210 | −22 | 430 | holds | 208 | holds |
| p-w pooled cut windows | 4,254 | 27 | 39 | +12 | 52 | holds | (none) | |
| DEV pooled cut windows (reading) | 900 | 1 | 2 | +1 | 5 | (reading) | | |

| At the free-decode opening (Q3) | n | a | b | b − a | V | Q3 |
|---|---|---|---|---|---|---|
| p-w s1 | 1,067 | 3 | 4 | +1 | 7 | holds |
| p-w s2 | 1,067 | 8 | 2 | −6 | 10 | holds |
| p-w s3 | 1,067 | 14 | 12 | −2 | 26 | holds |
| DEV pooled | 852 | 0 | 2 | +2 | 2 | holds |
| p-w pooled | 3,201 | 25 | 18 | −7 | 43 | holds |
| p-w pooled cut windows | 405 | 5 | 4 | −1 | 11 | holds |
| DEV s1, s2, s3 (readings) | 284 each | 0, 0, 0 | 1, 0, 1 | +1, 0, +1 | 1, 0, 1 | (readings) |
| DEV cut windows (reading) | 78 | 0 | 0 | 0 | 0 | (reading) |

- **Q4 is carried from claude-244,** and the runner recomputed it from the fields Q4's integer test reads. κ is 0.972, 0.903 and 0.919 on DEV and 0.856, 0.907 and 0.838 on p-w, against 4/9. The disagreements at a low margin (d_low of d) are 78 of 80, 42 of 46, 64 of 69, 356 of 409, 351 of 383 and 388 of 454.
- **On p-w's pooled counted rows, R is ahead:** by 22 under teacher forcing and by 7 at the opening. In three p-w strata it still loses net, each inside its bound: the cut windows under teacher forcing (+12), s2 under teacher forcing (+2) and s1 at the opening (+1). The card names no deciding panel, and every tested stratum on both panels holds.
- **Row 0 ran first and held in all six cells,** before any reading:
  - **I-Inputs:** 43 input files pass their sealed sums, and the four manifests equal the store's copies and the sheet's hashes.
  - **I-NoKey:** 0 keyed openings in R's six greedy files.
  - **I-Exit:** claude-244's sealed `grade.log` ends GRADE-DONE.
  - **The row identity:** positions are unique, and the prompt and reference agree across seeds. p-w has 135 cut-window rows per seed.
  - **The sampler code-path check** passes on the blob the sheet names.
  - **The probes** all behave as stated:
    - the good cell reads as hand-counted, and the checks-only pass holds no readings;
    - the empty-reply and missing-row cells fire;
    - all 7 decision cases decide as stated.
  - **claude-244's cell checks show 0 faults** in every cell. Over the six cells, claude-244's sealed `grade.log` counts 9,488 forced-greedy offsets and 8,629 aligned offsets checked. This run's log prints those counts only as booleans, by design (sheet §9, round 4a).
  - **I-Offset0 reproduces claude-243 exactly:** 4, 4, 5 and 64, 47, 78.
- **p-w's cut windows join.** I-Align held over their 405 rows, with 0 violations and 0 unchecked in 632 checked positions. So they join Q1 at every position, and their clear-margin disagreements at every position join Q5's set.

## What changed

- The prep sheet. It registers claude-245's bar verbatim, with Row 0, the sampler check by file and lines, the inputs and the grader by hash, and four FIT rounds and two pre-run rounds.
- [ORDER.md](ORDER.md), with the cancel probe's result, and this REPORT.
- **The item:** "Why it matters" records this grade. "What is missing" now names Q5's decision card as the next unit, and the item stays `ready`.
- One row in [m4-serving](../../../../research/notebook/m4-serving.md).
- No code changed. `docs/roadmap/README.md` was re-rendered and did not change.

## Measurements

- **The inputs** (the sheet's §5, by manifest hash):
  - R is claude-243's raw export. Its greedy replies are in claude-243's laptop seal.
  - F′'s rows and greedy samples are in claude-243's GPU run (`tie-parity/out/r-drop-sN/`).
  - R's forced decodes are in claude-244's seal.
  - The prompts are claude-232's six prompt files.
- **The grader**, in `local/claude/claude-246/`:
  - `grade.sh` `501306372773885974eaa7b6937a6f0aae839a5f0f4c02e26ac607e5061f3248`;
  - `grade-cell.jq` `6bac0aa087ff2db68743cef6340f326dcba1da16a5a8e9b822bebf7040029781`;
  - `decide.jq` `783a058c6c9ff6b615b5ab174205824bef800d6af871ca4ea395928b9e684300`;
  - `probe/decide-cases.json` `2275ed10710f1127a1976e9ee4fea8e03f4d05f9b3fa87e759e289779fd6e94b`.

  It ran once, with exit 0. It refused to start until the sheet's last lines read `TEST FITNESS: FIT` and `PRE-RUN REVIEW: PASS`, and while `grades/` or `grade.log` existed.
- **The sealed run:** `results/claude/integer-nearest-readout-grade-2026-11-01/`.
  - It holds 29 files and their `SHA256SUMS`: `grades/` (six checks, six cells and `decision.json`), `grade.log`, the grader and its probes.
  - Its `SHA256SUMS` is `e0a7f384842d16048747dbf5aec25ef89f078280dc21cae95a72679a05e2843c`, and `decision.json` is `7808259b8d250d1ce936e641565cd2c5bed75ed3f4e8a976ade683b1a829bbbe`.
  - All 29 read back from the store equal to their sums.

## Checks run at the head

- The sheet's Row 0, read from the run's files, as above.
- `loop gates --full`, `loop report --check` and `loop pr-check --local`: recorded in the review record at the exact head.

## Skills used

None.

## Still open

- **The next unit: Q5's decision card.** It fixes the trace's rule, meeting the four requirements in claude-245's card. Then a sheet with a FIT and a pre-run review, and then the trace. Its set is every clear-margin disagreement: 151 on p-w's counted rows and 11 on DEV's, plus the p-w cut windows' own at every position, since I-Align holds there.
- **Read F′'s own top two** at the disagreeing positions. Its sealed rows carry no scores, so claude-244's residual bound is one-sided.
- **Speed:** the nearest scan is 65% of a decode step (claude-243), so a cheaper exact nearest is still the speed lead.
- **Carried:** making raw the default or the snap faithful, and the leads in the item's "What is missing".

## Plain summary

The served integer model and the trained model were checked word by word against the reference reply, on every reply position of both test panels, at three seeds. The check counted how often only one of the two gets the word right. Under the bar fixed beforehand, the integer model is never measurably worse. Over the three seeds together, on the harder panel of new phrasings, it gets slightly more words right than the trained model. On the easier panel, it gets a few fewer, inside the allowance. The first word of a free reply reads the same way.

The bar has one part left: an exact trace of the disagreements where the integer model's choice was not close. That part needs its own rule first.
