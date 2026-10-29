# REPORT: one-model-end-to-end, claude-202 (step 1: the data census)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3407. The item stays `ready`, at step 2.

**Speed.** The census itself took 4.2 s and 1.1 GB on the M1 (release build), with no GPU. No serving change.

## What did not work, first

- **Neither panel can test new phrasings.** By the rule fixed in the ORDER before any count, both fail as held-out-template panels: the value-masked prompt-template share at K = 1,024 must be at most 0.5, and it is 0.642 for DEV and 0.665 for Fresh V2.
- **Three quarters of each panel's replies are verbatim TRAIN replies:** 0.750 for DEV and 0.777 for Fresh V2. For prompts: 0.608 and 0.639. Fresh V2's recorded "0 exact full TRAIN/DEV document overlap" is true of whole documents, and says nothing about turns.
- **So a large share of every score on these panels can be earned by recall.** That includes M1's bar (e), whose fresh-transfer rows come from Fresh V2.
  - The 0.750 counts replies, not scored tokens. Long replies are less often verbatim (reply 13-gram shares are 0.367 and 0.460), so the share weighted by tokens is lower and was not measured.
  - About a third of each panel's prompts (0.358 and 0.335) have masked forms TRAIN does not hold.
  - The M1 bar is Casey's call (#2028 `6507594237`).
- **A first attempt by a subagent wrote nothing.** The guard refuses subagent writes without a suborder, and the subagent's tool list alone filled about 140k of its context. The unit's code was written in the main session.
- **The value mask is a frequency proxy, not the generator's template.** It keeps TRAIN's K most frequent tokens and masks the rest and every digit.
  - **It cannot change the verdict.** The mask is a fixed function of each run, so the masked share is never below the exact share, and the exact prompt shares (0.608 and 0.639) already exceed the 0.5 bar. This was pointed out by the review.
  - Its shares also barely move between K = 256 and K = 1,024 (0.644 against 0.642 for DEV prompts).
  - A panel built from held-out templates, the next unit, does not depend on the proxy.
- **The 26,214,400 tokens and 10,975,281 targets** that the Fresh V2 sheet (`:62`) gives for TRAIN's sha are of unknown origin. They match neither the file's header nor its payload. That sheet's counts are a question for its own unit.

## What changed

- **`train-in-space --data-root DIR --fresh-v2 DIR --census 1`** (`in_space_long.rs`, `census`, and the binary's flag). One JSON line, holding:
  - M0, the sizes;
  - M1, the probe's pinned counts reproduced with its own `reply_table`;
  - M2, step 2's decision rows as DEV corpus positions;
  - M3–M6, per panel: exact, n-gram (8 and 13) and value-masked (K = 256 and 1,024) overlap, and opening keys;
  - the ORDER's new-template flag.
- **`reply_table` and `table_keeps`** are factored out of `reply_bound_rows`, with no change to its counts: its two tests pass, and M1 reproduces claude-192's numbers.
- **Fresh V2's pins** are new constants: the two shas, 44,678 tokens and 14,479 targets.
- **Results:** `docs/labs/one-model-end-to-end/census-2026-10-29.json` (counts and positions only) and its README, plus one row in the float-training notebook.

## Measurements

| | DEV | Fresh V2 |
|---|---|---|
| Replies equal to a TRAIN reply | 0.750 | 0.777 |
| Prompts equal to a TRAIN prompt | 0.608 | 0.639 |
| Reply 8-grams / 13-grams in TRAIN | 0.605 / 0.367 | 0.655 / 0.460 |
| Prompt 8-grams / 13-grams in TRAIN | 0.521 / 0.282 | 0.524 / 0.291 |
| Masked prompt templates, K = 256 / 1,024 | 0.644 / 0.642 | 0.668 / 0.665 |
| Masked reply templates, K = 256 / 1,024 | 0.781 / 0.766 | 0.810 / 0.798 |
| Openings keyed by TRAIN's first-8 table | 1,194 / 1,226 | 1,234 / 1,279 |

- **TRAIN:** 7,512,797 tokens (the header count; the file is 64 + 2 × 7,512,797 bytes), 60,000 documents, 212,968 prompts and replies. The 26,214,400 figure in the Fresh V2 sheet (`:62`) does not describe this file.
- **M1:** every pinned count reproduced exactly, as the README's table shows.
- **M2:** 3,406 unnamed later rows and 32 unkeyed openings, 3,438 positions in all.

## Checks run at the head

- **The census tests:** `cargo test --locked -p uor-r4-float-train --lib -- census reply_bound` gives 6 passed. They cover:
  - turn splitting at BOS and at mask changes;
  - n-grams that never cross a run;
  - exact whole-run matches;
  - value masks with digits and collapsed runs;
  - `digit_tokens`;
  - the decision-row selection.

  The two existing probe tests pass after the refactor.
- **The real run:** `--census 1` exited 0, with M1's numbers exact, as above.
- **`cargo test --locked -p uor-r4-float-train --no-fail-fast`:** 143 passed, 0 failed.
- **`uor-harness loop gates --full`**, at `44ad0ce4a` with this REPORT: green, 11 gates GREEN.

## The review

Fresh context, at `d8e88a2b8`, given the ORDER and the diff: **APPROVE WITH NITS**. It checked every headline number against the JSON with `jq`, and all are confirmed.
- **What it found sound:**
  - the turn splitting, the n-gram boundaries, exact matching and the digit masks (the real tokenizer has 11 single-digit pieces and no multi-digit ones);
  - the behaviour-preserving refactor;
  - M1, which uses the probe's own rows and tables, with 8,089 the correct maximum under the 0.83 rule;
  - M2's selection.
- **Its limit:** M1 checks the pins and the refactor. It is not an independent check of `reply_rows`.
- **The hash:** the two FNV passes are not independent. A false match could only raise a share, and its chance is about 1e-8 here.
- **Its findings, fixed at the next commit:**
  1. Two overclaims ("cannot test new phrasings", "mostly measures recall") reworded. The README and REPORT now also say the reply share counts replies, not tokens.
  2. The lower-bound argument added: the masked share is never below the exact share.
  3. The 26,214,400 / 10,975,281 figures recorded as of unknown origin.
  4. The test now has a frequent BOS and a tie to the smaller id. Removing `top_tokens`' BOS filter turns it red; checked here, then restored.
  5. Two definitions made explicit: the decision positions are target positions, and n-gram windows count repeats.

## Skills used

- None was invoked this unit.

## Still open

- **The next unit builds a panel with held-out templates,** and gives it its own census: held-out phrasings from the generator, or a held-out slice of its template space. The ORDER stops if no new-template panel exists.
- **Step 2,** the overturn check on DEV's 3,438 rows, can be prepared in parallel: its rows exist now.
- **The M1 bar** is Casey's (#2028).

## Plain summary

We counted how much of our test sets the training data already contains. About three quarters of the test replies appear word for word in training. So a large part of every score so far can be earned by remembering training replies, and none of our test sets is built from new wording. The next step is to build a test set whose wording the training data has never seen.
