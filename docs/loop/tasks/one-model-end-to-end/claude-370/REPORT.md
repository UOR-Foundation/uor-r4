# REPORT: one-model-end-to-end, claude-370 (rows addressed by the turn's token pairs, trained by the output error)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-pair-rows-2026-11-13.md](../../../../harness/prep/one-model-end-to-end-pair-rows-2026-11-13.md). PR #3961.

The order asked for one table of learned rows in the float trainer, addressed by the exact pairs of tokens of the user's turn (the product of the two tokens' primes), read as a sum at the reply's positions, trained by the trainer's own output error. It asked for four runs (two seeds, with and without the rows) on `wick`, scored in float on two panels against bars fixed before any model was trained. **The bar reads "Adds":** exact replies over the two panels together (2,218 rows) go from 431 to 527 on seed 2 and from 413 to 521 on seed 3. G is 4.60 points against a line of 3.5, each seed's gain is above zero, and replies that did not end rose by 0.02 points against a limit of 1. This is a float reading on panels. It is not a served model, and nothing here is a milestone claim.

## What did not work, first

1. **The facts the unit was aimed at barely moved.** The binding finding asked for "capital" and "Italy" together. Capital rows on P14: 1 and 0 of 20 without the rows, 0 and 1 of 20 with them; ended with the answer word: 6 and 2 to 4 and 5 of 20. On P23: 0 of 14 exact in every model. Day after: 1 and 0 of 15 to 1 and 1. Adding: 0 of 13 in every model; counting 0 of 12. The served reply is now close but broken ("The capital of Russia isososcow.", "isc."), not right.
2. **Most of the gain is copying, not knowledge.** On P14 the gain by need is B (a word of the turn) 24 and 27 to 73 and 70, almost all of it "repeat words" (19 and 21 to 66 and 63 of 105). Facts (C) gain 7 and 8; told values (T) do not move (102 and 93 to 101 and 100 of 604); never-told (N) is 0 of 79 in every model.
3. **The gain is larger on turns TRAIN never saw.** Exact replies on turns not in TRAIN: 49, 56, 47, 53 to 94, 91, 93, 93. On turns in TRAIN: 162, 164, 155, 158 to 169, 173, 171, 164. So the rows are not mainly memorizing TRAIN's answers. They help the model carry the turn's own words into the reply.
4. **The model leans on the rows.** With the rows cut from a trained `pair` model (I-8's control) P14 exact falls to 159 and 154: below the plain `b` models (211, 202). Replies that did not end rise to 183 and 310. The rest of the trunk learned around the rows.
5. **I-8's first pass was an instrument failure.** The scoring job copied the checkpoint without its pair-rows files into a folder named `norows-ck`, and the trainer's scoring pass refuses a checkpoint folder not named `step-N`. The sealed panel passes of the `pair` models were not touched. The control pass alone was run again by the same build (`00bd4c415…`), from a new root (`try-1/i8-RUN`, L-002), into `norows/step-016384`. It passed: 587 and 582 of 1,108 P14 replies change without the rows, and 1,108 of 1,108 scored replies read a bag on each panel. The sheet carries this as amendment 1.
6. **A count still runs on.** "Please count from 3 to 8." gets "3, 4, 5, 6, 7, 8, 9, 9, 9, …" on three of the four models. "What is jespaist?" loops "Jespaist is jespaist is …" on one `pair` model and one `b` model.
7. **Cost.** 187,413 pairs × 256 = 47,977,728 learned values, about 2.5 times the 19 MB artifact's whole size if exported as is. Training is 16% slower: 4,918–4,923 s for `b` against 5,842–5,846 s for `pair` at 16,384 steps (3.33 against 2.80 steps a second, two runs sharing one RTX 4090). Card memory at most 17.1 GiB against 18.7 GiB.
8. **Not run.** No serving or export. The integer export refuses a checkpoint with pair rows, by design. No `uor-chat` transcript. The GPU tests ran on `wick` inside the job's I-3 and I-4 checks at `00bd4c415…`. At the merge head `7575a10d1` only `cargo check -p uor-r4-float-train --bins --tests` was run on the laptop (pass, one dead-code warning).

## What changed

- **`crates/uor-r4-float-train/src/pair_rows.rs` (new):** the dictionary (every unordered pair of different tokens of a TRAIN user turn, address = product of the two tokens' primes, kept when seen in at least `MIN` turns), the rows (`pairs × width`, zeros at the start), the bag per reply span, their own optimizer, save and load with refusals of half a checkpoint, a damaged vocabulary and nested marks.
- **`geometric_stack_native.rs`, `in_space_long.rs`:** the bag is added once, at the first layer's seam, at every reply position. A cached step adds the bag the fill pass left for its row.
- **`bin/train-in-space.rs`:** `--pair-rows MIN`, a training-only switch. Without it nothing is created or saved.
- **`fbs_export.rs`:** the export refuses a checkpoint that holds pair rows.
- No panel, recipe, bar or saved model is changed.

## Measurements

Recipe: claude-358's `p` recipe, 16,384 steps, seeds 2 and 3. `pair` adds `--pair-rows 2`. Dictionary: 187,413 pairs, SHA-256 `0b0640280493d20d412d17cb2a16bebbd798bd8d6b13dfbf47042d046b69b82e`. At step 16,384 every row has a non-zero entry.

| model | P14 exact (of 1,108) | P23 exact (of 1,110) | did not end P14 / P23 | last train loss | last dev loss |
|---|---|---|---|---|---|
| b-s2 | 211 | 220 | 124 / 91 | 0.526 | 2.662 |
| b-s3 | 202 | 211 | 87 / 73 | 0.522 | 2.387 |
| pair-s2 | 263 | 264 | 96 / 92 | 0.458 | 2.370 |
| pair-s3 | 264 | 257 | 97 / 91 | 0.451 | 2.536 |
| pair-s2, rows cut (I-8) | 159 | — | 183 / — | | |
| pair-s3, rows cut (I-8) | 154 | — | 310 / — | | |

**The bar** ([bars.txt](bars.txt)): gains 96 and 108 of 2,218; G = 4.60 points; the lines were 155.26 whole replies for "Adds" and 66.54 for "Does not add". Replies that did not end: 215 to 188 on seed 2, 160 to 188 on seed 3, a rise of 0.02 points against a limit of 1. **Adds.**

**By kind on P14** (exact): other 133 and 136 to 188 and 190 of 520; value 79 and 67 to 76 and 75 of 509; not told 0 of 79 in every model. A reply that says "not told" where the reference is not told: 0 and 3 to 9 and 11.

**By need on P14:** [need-p14.txt](need-p14.txt). **Fact and number rows:** [subst.txt](subst.txt). **Gain split by TRAIN overlap:** [in-train.txt](in-train.txt). **Twelve replies word for word, the same rows for all four models:** [twelve.txt](twelve.txt). Three of them:

- "Guess what, I bought a jacket today." b: "A new camera! That sounds wonderful." / "A new guitar! …"; pair: "A new jacket! That sounds wonderful." (both seeds).
- "Which city is the capital of Russia?" b-s3 loops "The capital of Russia is the capital of …"; both `pair` models: "The capital of Russia is Moscow."
- "What is jindrendklut?" (told 2567) pair-s2: "It's fklut is 2567."; b: "It's f meet about the bike!"

Score files: [rows.sha256](rows.sha256) (the eight per-row files; kept in `local/claude/claude-370/read-1/`).

## Checks run at the head

| Check | Result |
|---|---|
| `cargo check -p uor-r4-float-train --bins --tests` at `7575a10d15a37825125e83519c4cd064f2babe1d` (laptop, CPU) | pass, one dead-code warning |
| The job's I-3 (no-flag smoke against the pin build) and I-4 (pair smoke, the checkpoint holds rows and dictionary) on `wick`, build `00bd4c415839956e163cfcba78d5b66a3c3c6d78` | pass (each stage EXIT 0) |
| I-8, the scoring pass reads trained rows | first pass: instrument failure (item 5); repaired pass: 1,108 of 1,108 replies read a bag on each panel, 587 and 582 replies change without the rows |
| The four training runs | EXIT 0, 16,384 steps each |

The crate's code has no diff between `00bd4c415…` (the build the GPU checks and all four runs used) and the head. The head adds only a merge of `main`, which touches no file of this crate.

**Review.** Two fresh-context reviewers (`delegate:reviewer`) read the head. Each was given the order's mechanism, its done-when tests and the code diff, nothing else. Both verdicts: **approve with low items**.

- `pair_rows.rs`: one medium item, "no test of the cached decode step in this file". It is closed: `the_pair_rows_step_as_the_whole_window_does` (`geometric_stack_native.rs:4009`) pins it.
- Low items, left open:
  - zero rows are equal in value, not in bits, at a residual entry of -0.0;
  - AdamW momentum keeps moving a row on steps that do not name it (its gradient there is zero);
  - a multi-token assistant header could hide a user marker (this recipe's marks are single tokens);
  - `load` checks neither the rows' dtype nor the dictionary's vocabulary against the model's;
  - the active rows live in a thread-local.
- Wiring: three low items.
  - The bag is added after layer 0's address memory, not before it. That matters only with `--address-memory`, which this recipe does not set, and nothing refuses the pair.
  - Only `Capture` and `Step` carry modes exist, so `ids` always holds the whole window. Checked, closed.
  - The export-refusal test pins behaviour that existed before this diff.

## Skills used

None.

## Still open

- **Serve it.** The rows need an integer form in the artifact and a read in `uor-chat`, then the `jacket` and `Moscow` replies read as served. At 48 M values the table needs a smaller form first: a higher `MIN`, or rows shared by an E8 decode of the pair.
- **Facts still need a sharper address.** A pair of frame words and a pair that holds the key word get the same share of the bag. The next unit should weigh rows by how few replies their pair is seen with, or add triples, and read the capital, day-after and legs rows.
- **Counting and number work** need a mechanism on the operands. No row keyed by tokens can do it.
- **The trunk without its rows is worse than without training them** (item 4): any later unit that drops or caps rows must retrain.

## Plain summary

The model was given a new table. For every pair of words in what the user typed, there is a row of numbers, and the model adds up those rows while it writes the reply. Training fills the table in.

On two panels of about 1,100 replies each, the model with the table gets about 105 more replies exactly right than the same model without it, on both seeds. That passes the bar set before training. Most of the gain is the model saying back the user's own words correctly ("A new jacket!" instead of "A new camera!") and repeating phrases it was asked to repeat. Facts such as capitals are now nearly right ("Moscow" appears) but are still mostly wrong letter for letter. Arithmetic and counting did not improve. The table is large, 48 million numbers, so it cannot be put on the laptop yet. That is the next step.
