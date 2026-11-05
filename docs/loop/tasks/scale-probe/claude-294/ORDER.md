# ORDER: scale-probe, claude-294

Lab: claude. Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472). Base: `9577aebc9`.

## Goal

The item's done-when reads:

> 1. **A prep sheet** `docs/harness/prep/scale-probe-<date>.md` with a fresh FIT and the D67 pre-run review, both posted on this item's issue. It registers:
>    - the arms, the cost and the stop rule;
>    - the panel, drawn fresh by `heldout-panel` at a new seed with its own census. P-dev stays reserved for `one-model-end-to-end`'s run;
>    - the bars, in advance.
> 2. **The runs,** sealed in the store with a SHA256SUMS that was read back. The pods are down.
> 3. **The REPORT** gives, per width and corpus size:
>    - training tokens per second, and served decode tokens per second against context on the exported artifact;
>    - the panel's reply scores, with paired bands across seeds;
>    - the fitted slope, and whether it is distinguishable from flat.
>
>    It also has one notebook row in `float-training.md`, and the pipeline map's gap 9 records the slope.

**This unit is done-when 1, the sheet.** The runs (done-when 2) and the REPORT (done-when 3) are the next units, under P5's bound of 1–2 run units.

- **Claim check, (a) the step:** Casey's plan of 4 November, P5, "The bigger train = `scale-probe`. [sheet 1 unit; run 1–2 units]", in the order decided on #2028 ([6634218223](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6634218223)): step 6, "P5 on real text", after the real-text corpus (done, claude-292 and claude-293) and P4 (done, claude-286 to claude-288).
- **Claim check, (b) the model:** no code changes. The sheet fixes what the next unit trains: the one model's recipe at widths 256, 512 and 1,024, on human-written text in place of the templated M-world, and the bars that say whether it gets better with width and data. That slope decides whether the trunk is scaled up next, which is the plan's question about reaching a GPT-4-class chatbot.

## Decision card (the Data line)

The decided order says: "P5's Data line changes to real text through a decision card in its sheet unit. The templated data stays as one contrast cell." This unit makes that card, on its own fresh decision review, logged on #3472 as `[claude] owner log:` and open to Casey's reversal:
- **Data:** the real-text corpus of claude-293 (OASST2 and Wikipedia, about 28M TRAIN tokens), at several fractions, in place of "the corpus at several fractions".
- **Panel:** the corpus's own held-out splits, cut by source id and read by no model yet, in place of a panel drawn by `heldout-panel`, which draws only templated M-world conversations. `heldout-panel` at a new seed still draws the contrast cell's panel. P-dev stays reserved.
- **Scores:** integer top-1 on held-out text against unigram and T2 count tables, with bytes per token under the pinned tokenizer. Bits per byte waits for a lawful likelihood (§4.8).

## Cancel probe (run first)

The unit is cancelled if any of these holds:
1. A `scale-probe` sheet or labs folder already exists on `main`.
2. Another open order or PR holds a file in this scope.
3. The real-text corpus is not sealed in the store with a `SHA256SUMS` read back.
4. The kill switch is on.
5. `depends_on: [one-model-end-to-end]` means the recipe is still undecided.

Result: to fill in.

## Files in scope

- `docs/harness/prep/scale-probe*`
- `docs/labs/scale-probe/**`
- `docs/roadmap/items/scale-probe.md`
- `docs/loop/tasks/scale-probe/**`
- `docs/model/pipeline-map.md`
- `docs/research/notebook/float-training.md`

## Done when

1. `docs/harness/prep/scale-probe-2026-11-05.md` exists in the project's sheet shape (`uor-harness check` passes its shape rule), with the Data line on real text and the templated contrast cell.
2. A fresh non-author FIT reads `TEST FITNESS: FIT`, posted on #3472 and recorded in the sheet.
3. A fresh D67 pre-run review reads GO (or GO WITH FIXES, fixed), posted on #3472 and recorded in the sheet.
4. The decision card is logged on #3472 as `[claude] owner log:`, and the item file's Data and panel lines say real text.
5. A REPORT, negatives first, and the in-unit exact-head review.

## Stop and ask if

Stop and decide, never stop and wait:
- **The trainer cannot take a cell as registered** (width 1,024, the corpus fractions, or the host memory the turn memory needs at 28M tokens): narrow the cell, write why in the sheet, and keep the three widths if the writer's cap allows them.
- **The projected cost passes the caps** (4 pods, $8/h): cut positions equally across cells, never drop a seed.
- **The FIT or the pre-run review finds a blocker:** fix the sheet and review the delta, at most two rounds.
