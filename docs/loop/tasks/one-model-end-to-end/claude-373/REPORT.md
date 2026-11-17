# REPORT: one-model-end-to-end, claude-373 (pair rows weighed by how few replies their pair is seen with)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-rare-pair-rows-2026-11-14.md](../../../../harness/prep/one-model-end-to-end-rare-pair-rows-2026-11-14.md). PR #3964. Follows claude-370 (PR #3961).

The order asked for claude-370's pair rows to be weighed by 1/R, where R is the number of different TRAIN replies the pair is seen with. It also asked for four models, trained as before at two seeds with and without the weight, and scored against bars fixed before any model was trained. **The bar reads "Does not add."** The fact and number rows that end with the answer word go from 85 to 75 of 188 at seed 2 and from 85 to 71 at seed 3, a mean gain of −12.0 against a "does not add" line of under 4. Exact replies over the two panels also fall, by 141 and 68 of 2,218, against a guard of 33. This closes one configuration: 1/R, R capped at 4,096, no other scale. It does not close weighing pair rows. It is a float reading on panels, and nothing here is a milestone claim.

## What did not work, first

1. **The weight made the facts worse, not better.** Fact and number rows ended with the answer word, P14 + P23, of 188: `pair` 85 and 85, `rare` 75 and 71. Capital (34 rows): exact 0 and 2 for `pair`, 0 and 0 for `rare`. Day after (28): 4 and 4 to 0 and 2. Add (21): 2 and 0 to 0 and 1. No fact kind gains at both seeds.
2. **The copying gain claude-370 found is mostly lost.** Repeat words, 212 rows: `pair` 118 and 125 exact, `rare` 24 and 67. By need, B (a word of the turn), of 276: 133 and 142 to 36 and 80. That is most of the exact loss (141 and 68).
3. **The cause is visible in the replies: `rare` copies, then does not stop.** At seed 2 on P14:
   - "Say exactly this: peebrarfais choutmout klerkteel". `pair`: "peebrarfais choutmout klerkteel". `rare`: "peebrarfais choutmout klerkteel Assistant: peebrarfais choutmout klerkteel Assistant: peeb".
   - "Say exactly this: sokpeestdir keerkwaismoust vustleel husreest beelnod". `rare` copies all five words, then "relievedont day after after after after …".
   - "Say exactly this: wushodeer … ramdark". `rare`: the seven words right, then "Assist".

   Replies that did not end, of 2,218: `pair` 187 and 210, `rare` 227 and 222. On P14 alone, at seed 3, `rare` is right on 13 rows where `pair` is wrong and wrong on 52 where `pair` is right.
4. **Reading: frame-word pairs carry where the reply ends.** A pair such as (say, exactly) is seen with thousands of different replies, so 1/R scales its row to almost nothing. That is the pair claude-370's rows learned the reply's shape from, including where a copy stops. Content pairs keep their share, and the content of the copy survives. This is the reading of items 2 and 3, not a separate measurement.
5. **Instrument slip 1: both lanes sealed `pre` to one store path.** Lane b's `pre` seal was refused because the path already held lane a's (seal exit 2). Only lane a's `pre` record is kept: the build, the GPU tests and the I-4 census. Lane b's chain goes past `pre` only when its own `pre` passed I-1, I-6 and I-4. Lane b's training census for `rare-s3` equals lane a's I-4 census character for character. Both lanes' binaries are byte-identical (below). Nothing read here rests on the missing record. The next kit must put the lane in every sealed path.
6. **Instrument slip 2: the reader breaks when OUT is a relative path.** read-373.sh's staging links are relative, so `subst-373.sh` found no rows and the first read stopped with "subst-373.sh failed", writing no verdict. It was run again with absolute paths into a new folder (`run/read-try-1b`) by the same reader, SHA-256 `2c02c22d05f6fcccee37191ff89ed478be2fab7d4f81e60c32349a994446ab46`. Nothing in the reader changed.
7. **Departure from the order: R is capped at 4,096.** The order says 1/R. The code stores min(R, 4096), and the census prints the cap. The census max of 4,096 shows the cap is reached: the most common pairs weigh 1/4096 rather than less. The cap was set before any run, not chosen from a result.
8. **Not run.** No serving or export (the integer export refuses pair rows, as in claude-370). No third seed. No arm, seed or step count was added after reading.

## What changed

- **`crates/uor-r4-float-train/src/pair_rows.rs`:**
  - R per kept pair: the number of different TRAIN reply runs among the turns that hold it, counted in the dictionary's own pass and capped at 4,096. The weight is 1/R, saved with the rows.
  - A census line, `N pairs; R min … p50 … p90 … p99 … max … (cap 4096)`.
  - Refusal of half a checkpoint (one of its two files missing). A rows file that lost its `weights` tensor still loads, unweighted: in this run only I-8's `none` line would catch it (Still open).
  - Tests: R counts each different reply once; a pair seen with one reply weighs 1; the bag is the weighted sum; without the flag it is claude-370's, bit for bit; save and load; cached decode equals the whole window.
- **`in_space_long.rs`:** the bag reads the weights when they are present.
- **`bin/train-in-space.rs`:** `--pair-weight replies` (with `--pair-rows MIN`). Training prints `[pair] weight replies: <census>`. Scoring prints `[pair] loaded weight replies: <census>`, or `none` for unweighted rows (I-8).
- No panel, recipe, bar or saved model of another unit is changed.

## Measurements

Every model: claude-358's `p` recipe, 16,384 steps, `--pair-rows 2`, plus `--pair-weight replies` for `rare`. All four ran on RTX 5090 pods (`vast-60613879` lane a, seed 2; `vast-60614229` lane b, seed 3), built from `d7c9155766c74a5dde67395bffb4b6a63808fd25`. Scored in float by the trainer's own pass on P14 (1,108) and P23 (1,110).

| model | exact P14 | exact P23 | exact of 2,218 | did not end | fact rows ended with the answer word, of 188 |
|---|---|---|---|---|---|
| pair-s2 | 258 | 261 | 519 | 187 | 85 |
| rare-s2 | 190 | 188 | 378 | 227 | 75 |
| pair-s3 | 260 | 260 | 520 | 210 | 85 |
| rare-s3 | 221 | 231 | 452 | 222 | 71 |

Bars: gain −10 (seed 2), −14 (seed 3), mean −12.0. Guard: pair − rare exact = 141 and 68 (limit 33). **Does not add.**

The control reproduces claude-370's `pair`, which had exact 527 and 521 on the RTX 4090 `wick`. Here it is 519 and 520 on the 5090.

By need, exact of 2,218 (pair-s2, rare-s2, pair-s3, rare-s3):
- A, 342: 126, 121, 122, 124.
- B, 276: 133, 36, 142, 80.
- C, 156: 32, 27, 37, 26.
- D, 67: 13, 10, 11, 11.
- N, 163: 0 in every model.
- T, 1,214: 215, 184, 208, 211.

By kind, the rows that move (same order):

| kind | of | pair-s2 | rare-s2 | pair-s3 | rare-s3 |
|---|---|---|---|---|---|
| repeat words | 212 | 118 | 24 | 125 | 67 |
| told value | 1,214 | 215 | 184 | 208 | 211 |
| day after | 28 | 4 | 0 | 4 | 2 |
| colour | 19 | 8 | 6 | 9 | 6 |
| sentence with a word | 17 | 4 | 1 | 5 | 2 |
| capital | 34 | 0 | 0 | 2 | 0 |

The other kinds (goodbye, thanks, legs, sound, greeting, first letter, bigger, …) move by at most 4 in either direction.

**Capital replies, P14, seed 2, word for word.** Both arms fail the same way:
- "Which city is the capital of Norway?" gets "The capital of Norway is the capital of Norway." from both.
- "What's the capital city of Kenya?" gets "The capital of Kenya is is is is …" from `pair` and "The capital of Kenya is isistant: The capital of Kenya is isistant: …" from `rare`.

**Weight census** (I-4, TRAIN, the same in the `rare` training logs and in the `rare` scoring loads): 187,413 pairs; R min 1, p50 6, p90 48, p99 545, max 4,096 (cap 4,096). The `pair` scoring loads read `none`.

**Speed.** Training seconds for 16,384 steps:
- `pair`: 2,681 and 2,671 (6.11 and 6.13 steps a second).
- `rare`: 2,657 and 2,657 (6.17).

The weight costs nothing measurable. Card memory peaked at 4,496 MiB for every model.

**Identity.**
- Binaries, the same on both pods: `train-in-space` `4b3505976e61324cf05621e53b9095f47bbc160dcde5d90974bc6c103d3600b8`; `fbs-snapshot` `24886a92d04884fde66653d94d91ec52b01721ec1043c18b720ae14757426f3d`.
- Toolchain: cargo 1.97.1, rustc 1.97.1, nvcc 12.8.
- Dictionary SHA-256: `0b0640280493d20d412d17cb2a16bebbd798bd8d6b13dfbf47042d046b69b82e`, the same as claude-370's.
- `subst-373.sh` differs from claude-370's `subst-370.sh` only in its name, the reader's name in its comment, and the four model names it reads.

## Checks run at the head

| Check | Result |
|---|---|
| Lane a `pre`, at `d7c915576`: GPU tests of `uor-r4-float-train` | 576 passed, 0 failed, 18 ignored (28 pair-rows tests among them) |
| I-1 (inputs' hashes), I-6 (pin), I-4 (R varies: p90 48 > 1, max 4,096 > p50 6), both lanes | pass |
| I-8, every scoring pass reads the weights it was trained with (`rare`: census equal to training; `pair`: `none`) | PASS at all four score stages |
| Each `rare` training census equals I-4's, compared on the laptop | equal, character for character, at both seeds |
| The nine stages' SHA256SUMS against the fetched tarballs on the laptop; gate-down before each pod went down | all match; both pods down after the gate (exit 0) |
| Harness `the_real_sheets` and `uor-harness prep-check` on the sheet at `caf75f851` | pass; `FIT FIT yes` |

**Review.** The run sheet passed its FIT on the fourth fresh read. Each of the three reads before it found a different way a broken measurement could pass, and each was fixed before compute. The pre-run review gave GO. This REPORT and the code at the head take a fresh-context exact-head review before the queue.

## Skills used

None.

## Still open

- **Weigh shape and content apart.** Frame pairs carry where the reply ends (items 3 and 4). A next configuration keeps them at full weight for the end decision, and weighs only the content rows by 1/R. One concrete form: two bags, the plain sum (claude-370) and the 1/R sum, each with its own seam. The other form: a floor on the weight. The order's mechanism is not closed by this result.
- **Facts are still not reached by pair rows in either form.** Capital stays at 0 to 2 of 34. The address may need triples, or the key word's own row.
- **`load` should refuse a checkpoint whose config says weighted but whose rows file has no `weights` tensor** (review, low). It loads unweighted today.
- **The kit must put the lane in sealed paths** (item 5), and the reader must refuse a relative OUT or make absolute links (item 6).
- **Counting and number work** still need a mechanism on the operands.

## Plain summary

claude-370 gave the model a table with one row of numbers per pair of words the user typed, and the model got better at saying back the user's own words. This unit tried to make the rare, informative pairs count more. Each row was scaled by one over the number of different answers its pair appears with, so that "capital, Italy" would count more than "what, is".

It made things worse. Facts did not improve (about 10 fewer of 188 at both seeds), and the copying gain mostly disappeared. The replies show why. The model still copies the right words, but it no longer knows when to stop, and it runs on into "Assistant: …" or repeats a word. The common pairs this scaling shrank are the ones that told the model where a reply ends. The next try keeps those pairs at full strength and scales only the content pairs.
