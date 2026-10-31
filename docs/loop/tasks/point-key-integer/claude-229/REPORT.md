# REPORT: point-key-integer, claude-229 (the run, on the laptop): row 0, an instrument fault

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses on #3436's false glob overlap. Sheet: [point-key-integer-run-2026-10-31](../../../../harness/prep/point-key-integer-run-2026-10-31.md).

## Tokens per second

**Not reported for the served model.** The decodes ran the native writer, which is not the trained model's readout (below). The served decode rule changes in the next unit: whole-table nearest adds a 4,096-row scan to every token. So the wall times here (in `log/decode-*.time`) time the wrong decoder, and done-when 5 is read after the repair.

## What did not work, first

- **The integer artifact does not decode as the trained model does: row 0, an instrument fault.**
  - **The grade read nothing.** Templates and exact replies show (0, 0) discordant pairs in every split and seed, and `carries` is false everywhere, p-w included, where claude-220's float key read (50, 9), (44, 4) and (47, 5).
  - **Integer greedy against claude-220's float greedy** on the same prompts at seed 1, memory on:
    - DEV: the first tokens agree on 0 of 310 prompts. The float's first token equals the reference on 129, the integer's on 0.
    - p-w: they agree on 2 of 1,202. The float's equals the reference on 179, the integer's on 4.
  - **The likely cause, named by the code and not yet tested by a run:**
    - (r-drop) trained with `objective: "cell"`, which `in_space_long` decodes as the whole table's nearest token (`decodes_nearest`).
    - The integer session serves the native writer, and computes nearest only beside it for timing (`stack_whole_table_nearest`).
    - PLACE.md's table has carried this gap since 29 Oct ("the served session still reads the native writer"). The planning of claude-225 through claude-229 did not re-read it.
- **The sheet's instruments could not see it.** Every check compared the integer artifact with itself (served against greedy, the search against a reference scan), or the trunk's writer token with the GPU's writer token (claude-227). None compared integer greedy with the trained model's own decode. Three FIT rounds did not ask for that check either.
- **Nothing is counted, and form C's bar is untested in integers.**
  - p-x was decoded and graded under the fault. Its reading carries no information about the mechanism: neither arm made a single template or exact match.
  - The sheet's row 0 ("Repair in a new unit, then rerun") keeps p-x as the deciding panel.
- **The CI check "harness rules and tests"** fails on DeepSeek's three documented-command REPORTs on main, not on this unit.

## What worked

- **Done-when 2's instruments held throughout:**
  - every served opening matched the reference scan: 8,295 openings, `reference_checked` = `keyed` in all 24 decodes;
  - each `/8` table read back as built, with 212,968 rows (`I-Rows`);
  - no BOS was named;
  - every artifact's sha256 equals its printed one, and `/7`'s `rec_gates` is `geometric-retention`;
  - the snapshots matched the store, each retained once, which closes claude-227's open check.
- **The fresh panel p-x** (seed 2131, 1,200 conversations, 149,800 tokens): a byte-identical second draw, stored with its sums, and a census in which DEV's and Fresh V2's rows reproduce p-w's census exactly.
- **The point table's build, on the laptop:** about 2,010 s a seed with the three in parallel, as a 200,000-token slice projected (51.7 s for 5,680 openings).
- **The prompts:** 310, 1,202 and 393 per seed, as claude-224 found, and p-x at 1,204.
- **Resident memory:** about 190.6 MB for greedy and 410 MB for served.

## What changed

- `docs/harness/prep/point-key-integer-run-2026-10-31.md`: the sheet, with its three FIT rounds and the pre-run review.
- `docs/labs/point-key-integer/README.md` and `census-p-x-2026-10-31.json`: the lab record and p-x's census.
- `docs/research/notebook/float-training.md`: one row.
- `docs/roadmap/items/point-key-integer.md`: "The units" records the run's row 0, and unit 5 is the repair.

## Measurements

- **The store:** `results/claude/point-key-integer-run-2026-10-31/`, 101 files under `SHA256SUMS`, listed back. p-x is at `inputs/claude/heldout-panel-p-x-2026-10-31/`.
- **The artifacts:**
  - `/7`: `19174e5e…`, `a3065ffb…` and `22192fef…`, 51,310,848 B each.
  - `/8`: `47fb4633…`, `7713ffbf…` and `b06cd97a…`, 161,202,592 B each.
- **The capped counts** (greedy / served) on p-x: 118 / 117, 116 / 118 and 118 / 118. On p-w: 136 / 135, 136 / 135 and 136 / 136.

## Checks run at the head

- The sheet's §4 checks, each read from the run's own files, as above.
- `the_real_sheets_pass_the_shape_rule`: passes locally.

## Skills used

None.

## Still open

- **Serve the trained model's readout in the integer artifact** (the next unit, under its own item):
  - whole-table nearest for a cell-objective model, flagged in the artifact from the run's objective;
  - checked by end-to-end token parity of integer greedy against claude-220's stored float greedy on DEV and p-w, memory on. That needs no GPU, and it also covers the turn memory and the Hamming read on a trained model, which nothing has checked yet.
- **Then rerun this sheet on p-x,** with the repaired `/7` and `/8`. The point table is teacher-forced, so it does not depend on the readout.
- **A lesson for every integer sheet:** add an instrument check that the integer artifact's greedy decode reproduces the trained model's greedy decode, before any grade is read.

## Plain summary

The checks built in the last unit all held. But the converted model did not answer like the trained one: it picked its words with the wrong final step. The trained model chooses the nearest word in its vocabulary, while the converted model used a different picker. So the test answered nothing either way.

Next, the converted model gets the same final step as the trained model. It will be checked word for word against the trained model's own answers, and then the test reruns.
