# REPORT: integer-nearest-readout, claude-244 (the bar's margin: R's teacher-forced agreement with F′ and its own top-two margin): near-tie residual

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Sheet: [integer-nearest-readout-margin-2026-11-01](../../../../harness/prep/integer-nearest-readout-margin-2026-11-01.md), FIT in round 3 after two NOT_FIT rounds (6561408154). References #3462.

## Tokens per second

No new decode speed is claimed. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1, on this M1.
- The new `--forced` mode stepped 312 positions/s on DEV (4,447 positions in about 14.2 s) and 250 on p-w (15,349 in about 61.4 s).
- That rate includes each prompt's prefill, the whole table's top two at every offset, and the per-offset check that the step's token is the nearest. It is an instrument's rate, not a serving reading.
- **Peak resident memory:** 190,660,608 to 190,709,760 bytes per decode (`/usr/bin/time -l`), the same as claude-243's greedy decode.

## What did not work, first

- **The bar is not met, and this unit does not test it.** claude-243's p-w first-token MISS stands, and the item stays `ready`. This unit measures where the gap sits; it changes no bar.
- **151 of the 1,246 pooled p-w disagreements (12.1%) are at clear margins.** They sit above their cell's G₁₀, up to g = 125,797 (Q12 units of −L1). A small residual at their own position does not explain them.
- **p-w seed 3 alone is over 3% at offsets ≥ 1:** r₊ is 397 of 12,864 (3.09%). The pooled r₊ is 2.89%, at or under 3%, and the sheet registered the pooled value as the one that decides.
- **F′'s own margin is not measured.** F′'s sealed rows carry its `nearest` but no scores. So the reading is one-sided: it places the disagreements at R's near ties, and bounds the residual there (g ≤ 2ε), but does not read F′'s top two.
- **The cut windows disagree more at the opening.** They are left out of the counts by the sheet. Their first tokens disagree on 23, 13 and 21 of 135 p-w rows per seed (10% to 17%). The counted rows disagree on 41, 34 and 57 of 1,067 (3.2% to 5.3%). This is a reading, with no bar.

## What worked

- **The reading is the near-tie residual row, on p-w (which decides) and on DEV.** R's disagreements with F′ sit where R's own top two are nearly tied:

| Pooled over s1–s3 | Disagreements d | At g ≤ G₁₀: d_low | c | q | κ | Clear of chance (3 SE) | Kind |
|---|---|---|---|---|---|---|---|
| **p-w** (decides), of 41,793 positions | **1,246** | **1,095** | **0.879** | 0.100 | **0.865** | yes (SE 0.0085) | **near-tie residual** |
| DEV, of 12,441 positions | 195 | 184 | 0.944 | 0.100 | 0.937 | yes (SE 0.021) | near-tie residual |

- **Every seed reads the same way.**

| Cell | d₀ of n₀ | d₊ of n₊ | G₁₀ | c | κ |
|---|---|---|---|---|---|
| p-w s1 | 41 of 1,067 | 368 of 12,864 | 10,112 | 0.870 | 0.856 |
| p-w s2 | 34 of 1,067 | 349 of 12,864 | 8,949 | 0.916 | 0.907 |
| p-w s3 | 57 of 1,067 | 397 of 12,864 | 7,760 | 0.855 | 0.838 |
| DEV s1 | 4 of 284 | 76 of 3,863 | 30,228 | 0.975 | 0.972 |
| DEV s2 | 3 of 284 | 43 of 3,863 | 22,526 | 0.913 | 0.903 |
| DEV s3 | 4 of 284 | 65 of 3,863 | 24,022 | 0.928 | 0.919 |

- **The disagreements sit at margins 80 to 100 times smaller than typical.** On p-w the median g at a disagreement is 1,778, 1,635 and 1,568, against a median over all positions of 183,087, 145,694 and 128,152.
- **No pick anywhere was an exact tie** (z₀ = 0 in all six cells). The ties-to-the-smaller-id rule never decided a pick.
- **The gap is per position, not special to the opening.** On pooled p-w, r₀ = 132 of 3,201 (4.12%) and r₊ = 1,114 of 38,592 (2.89%). That is 1.43×, under the sheet's 2×. On DEV, r₀ is 1.29% and r₊ is 1.59%.
- **The sheet's next unit follows from its table:** near-tie residual with pooled r₊ ≤ 3%, so a decision card re-derives the first-token bar from this measured per-position residual. No start is named for a card.
- **Every instrument check held in all six cells.**
  - I-Inputs: 27 of 27 sums.
  - I-NoKey: 0 keyed openings.
  - I-Forced-Greedy: 0 violations in 9,488 positions checked.
  - I-Align: 0 violations in 8,629 positions.
  - I-Coverage: every position in F′'s rows, with its target.
  - I-RunnerUp: 0 wrong.
  - Negative margins: none.
  - I-Exit: all six decodes exited 0, none capped.
- **I-Offset0 reproduces claude-243 exactly:** 4, 4, 5 and 64, 47, 78. The counted openings plus the cut windows' openings give the same numbers, 41 + 23, 34 + 13 and 57 + 21 on p-w. So F′'s teacher-forced opening equals its greedy first token at every opening, as it must.
- **The grader's probes** passed before any decode: the hand-counted cell, the checks-only pass, the bad cell, and 7 of 7 pooled decisions.

## What changed

- `stack/session.rs`: `IntegerStackModel::whole_table_nearest_two`, the whole table's nearest two with their `−L1` scores. Tests T3a (against a brute scan and the served pick) and T3b (ties).
- `fbs-toks.rs`: `--forced PROMPTS_JSON --out FILE`. It refuses a point-key artifact and a non-nearest head, and checks at every offset that the step's token is the nearest. Tests T1, T2, T4 and T5, with negative gates for the feed and the runner-up.
- The prep sheet, with three FIT rounds recorded in its §9.
- [ORDER.md](ORDER.md) and this REPORT.
- The item's "What is missing", which now names the decision card. It stays `ready`.
- One row in [m4-serving](../../../../research/notebook/m4-serving.md).
- `docs/roadmap/README.md` was re-rendered and did not change.

## Measurements

- **The inputs:** R is claude-243's raw export, `R/a7/a7r-sN.uorlut`. F′ is claude-243's sealed GPU rows and greedy samples, from `tie-parity/out/r-drop-sN/`. The prompts are claude-232's six prompt files.
- **The decodes:** `fbs-toks C243/R/a7/a7r-sN.uorlut --forced C232/prompts/prompts-SPLIT-sN.json --out R/forced/forced-SPLIT-sN.json`, run one after another from `local/claude/claude-244/`. They took 13.95 to 14.29 s per DEV cell and 61.21 to 61.47 s per p-w cell, with load averages of 1.6, 2.4 and 2.7 (1, 5 and 15 minutes) read just after.
- **`fbs-toks`:** `b3d6c601…`, built at the head (`c5af11201`, crates unchanged since `45397f50b`).
- **The grader:** `grade.sh` `bb248a0d…`, `grade-cell.jq` `984e2fe9…`, `pooled.jq` `12ad3d2a…`.
- **The sealed run:** `results/claude/integer-nearest-readout-margin-2026-11-01/`. It holds the six forced decodes, their logs, the checks and readings per cell, the pooled readings, `grade.log`, the grader and its probes. It has its own SHA256SUMS, and all 45 read back equal from the store.

## Checks run at the head

- The sheet's §4 checks, each read from the run's files, as above.
- The `--forced` and `whole_table_nearest_two` tests and their negative gates, and the integer library suite.
- `loop gates` and `loop pr-check`, as recorded on the PR.

## Skills used

None.

## Still open

- **The next unit: the decision card, as the sheet's table names it.** It re-derives the first-token bar from this measured per-position residual. That needs my own fresh decision review, logged on #3462. It must state how it treats p-w s3's 3.09% and the 151 clear-margin disagreements.
- **Trace the clear-margin disagreements** (g > G₁₀, 12% on p-w), layer by layer: trunk, Hamming read, memory, readout.
- **Read F′'s top two** at the disagreeing positions, so the residual is bounded from both sides.
- **The cut windows' openings** disagree about three to four and a half times as often as the counted ones. That is a reading for the localization, with no bar yet.
- **Speed:** the nearest scan is 65% of a decode step (claude-243), so a cheaper exact nearest is still the speed lead.
- **Carried from earlier units:**
  - making raw the default, or the snap faithful;
  - the read's selection by counting, then a re-time;
  - claude-240's code minors;
  - claude-241's Lows;
  - the long_sweep_prompt vocabulary check.

## Plain summary

When the integer model and the trained model disagree on the next word, the integer model was almost always close to a coin flip between its top two choices. On the panel of new phrasings, 88% of the disagreements fall in the 10% of positions where its top two were closest. Small rounding differences from the conversion would produce that pattern; a large systematic difference would put many more of them where its choice is clear.

The disagreements are spread over every position of the reply, not stuck at the first word. A small share, 12%, are not near a tie, and they still need tracing. The next step is to set the parity bar from this measured rounding effect, in a decision card, rather than from an older estimate.
