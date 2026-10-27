# REPORT: pipeline-map, claude-148 (the model end to end: each stage's domain, size, hardware and cost)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [pipeline-map](../../../../roadmap/items/pipeline-map.md), opened by claude-147 (#3333, merged as `3fa93b3b9`).

## What did not work, first

- **No new measurement.** The optional M1 decode measurement was not run. The page cites claude-126's `fbs-toks` run on this same laptop, and that run is not a qualified serving rate, because the artifact it timed departs from its specification.
- **Some figures are proofs or estimates, not measurements.** The page labels each one:
  - The trunk's size, about 64,000 trainable values, is arithmetic on `shapes()`. No receipt prints the count.
  - The serving-speed gap, about 5,000 times below the laptop's bandwidth limit, assumes cost grows with values read. The FBS session has never been profiled.
  - The FBS artifact's size is not on any line read for the page, and the page says so.
- **The most promising look-back form is not on main.** claude-146's Hamming read is on `claude/hamming-read` (#3332), and its result is not recorded. The page cites the branch and marks the stage as a hypothesis.
- **The external figures were not re-fetched in this session.** The page labels them external and names each publication, so a reader can check them at the source. They are: Apple's M1 figures, the GPT-4 report and announcement, `cl100k_base`, Llama 2, Llama 3, Phi-3, Chinchilla, and Allen-Zhu and Li.
- **Two slips of mine, caught before review by re-reading the sources:**
  - I first took 21,820–21,989 tok/s as the rate of each of two arms sharing a GPU. That range is the read arms alone; their twins ran at 24,852–25,068. The GPU figure is now about 47,000 tok/s, and the training arithmetic follows from it: 1T tokens on 4 GPUs in about 62 days and $3,300.
  - I first wrote that neither main look-back read cleared its openings bar. The content read did clear it at one seed of three (302 against 265), and failed reach at all three. The page now says both reads were negative against their bars as a whole.
- **`architecture.md` stays out of date beyond the pointer line.** The scope allows that file one pointer line only. The new line says the page covers the discrete path only and is partly stale.
- **`uor-harness check` has one error, in a file this PR does not touch:** `docs/harness/prep/m1-close-saved-model-2026-10-26.md` (`prep.sheet_shape`). The same error is on main.

## What changed

Size: one new page of 276 lines, a REPORT and an ORDER, and five small edits.

- **`docs/model/pipeline-map.md` (new).** It covers:
  - the two model paths, and what they share;
  - a one-table chain of the eleven stages, with each stage's input and output domain and whether the next stage reads it;
  - for each stage, a seven-field table: mechanism, domain, size, hardware, cost per token, status, and dependencies;
  - the scale arithmetic against published reference models, with the geometry's claims split into measured and hoped for;
  - ten gaps, ranked by what blocks the goal first;
  - an order of work, U1–U9, each with what a pass and a fail unlock;
  - a proposal for the owner's queue that relaxes nothing;
  - the outside sources.
- **`docs/SUMMARY.md`:** one entry under "The model, end to end".
- **`docs/model/architecture.md`:** one pointer paragraph.
- **`docs/roadmap/items/pipeline-map.md`:** `status: done`, and the round-2 reviewer's wording nit on #3333, so the first form's cause reads as the research note's reading.
- **`docs/roadmap/README.md`:** rendered.
- **`PLAN.md`:** the 27 Oct direction row's note now links the map. The row stays `open`, because the order of work carries the steps still missing.

## Measurements

None. This unit makes no measurement. It cites and checks existing ones.

## Checks run at the head

- **`cargo xtask links`:** 1,191 broken links repo-wide, the same count as on `5bc6f6522`, and none in this PR's files. The page's links were checked after staging; the checker reads tracked files only.
- **`cargo xtask roadmap check`:** 97 items OK.
- **`cargo xtask roadmap render --check`:** current.
- **`uor-harness check`:** 211 rules, 1 error, in the untouched file named above.
- **Claim wording:** each number on the page carries a source line or a proof, hypothesis or external label. A fresh-context review checks this at the exact head; see the PR's review record.

## Skills used

- **holding-the-goal.** The owner asked for every step, with its sizes, hardware and the fastest lawful mechanism. Where the repository has no answer, the page names the gap and a unit for it rather than an estimate shown as fact.
- **solution-integrity.** Both of my slips above were found by re-reading the rows I cited, before review, and corrected in place.

## Still open

- **The note for Casey** on #2028, after the merge: the page's link, the five biggest gaps and the first unit.
- **The first new unit is U2, one decoder.** It trains with the existing `voronoi` objective past the writer's collapse point, and reads the writer and whole-table nearest on the same checkpoints. It needs its own item, sheet and FIT.
- **U1 is claude-146's,** in flight on #3332.

## Plain summary

This page lays the model out as a chain of eleven steps, from the training text to the words a laptop prints. For each step it says what goes in, what comes out, and whether the next step can use what comes out. In two places the next step cannot. The memory store returns facts that nothing reads, and earlier attempts to look back at the prompt copied words to where the answer-picker never looks. The page also compares the model with published models: it is about 3,400 times smaller, it has seen about 440,000 times less text, and per number it reads, it runs about 5,000 times slower than this laptop's memory could feed it. The first fix is to make the step that picks each word the same in training and in use.
