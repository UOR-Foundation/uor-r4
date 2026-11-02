# ORDER: one-model-end-to-end, claude-259 (the trained-as-served run's sheet)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `22fecde5a`.

## Goal

Casey, 2 Nov 2026: "Stop Q5 now (Recommended)", to "Stop the Q5 track now and move to one model trained on the exact values it is served with?". Before it: "I thought we had agreed on returning to the end-to-end same-value trained as served method?"

**Plan step (the claim check):** PLAN.md's 29 Oct direction, step 4, "One model trained with all of them, exported, served on the laptop and scored on prompts, with tokens/s". **What it changes in the model:** nothing yet. It fixes, before the run, the recipe, the bars and the decision table of the run that trains the model to ship on its served values. claude-258 (PR #3556, merged as `22fecde5a`) built `train-in-space --train-as-served`; unit 3 runs what this sheet fixes.

## This unit

Unit 2 of 3 in the bound recorded in the lab's place record:
1. claude-258, the build;
2. this unit: the prep sheet, its FIT and its pre-run review, for (r-drop) trained as served at 3 seeds;
3. the GPU run, the export, laptop serving, M1's bars and tok/s. Then back to Casey with the result.

No trace, audit or new instrument in this track.

## Cancel probe (run first)

1. Is there already a sheet for a trained-as-served run? (`grep -rln "train.as.served" docs/harness/prep/`)
2. Which (r-drop) recipe is the one to ship? The 29 Oct training sheet's, or the 30 Oct cut-train sheet's with `--reply-read-train all`? Which did the served model claude-220 and its successors used?
3. Are the inputs in the HF store: the driver data, and the float-trained (r-drop) checkpoints at 3 seeds as the float control? Does the export and laptop serving path take a checkpoint whose `config.json` holds `"train_as_served": true`?

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **The sheet**, `docs/harness/prep/one-model-end-to-end-trained-as-served-2026-11-02.md`, one page. It holds:
   - the question;
   - the arms, each defined exactly: (r-drop) with `--train-as-served 1` at seeds 1–3, and the float control;
   - M1's existing bars and a decision table, where every outcome changes a decision;
   - the instrument checks: every seed's export-identity check (a trained step that rounds as exported must be unchanged), and a two-arm pre-run smoke at the run's head that runs `run_long` itself (claude-258's review, finding 1);
   - the data and checkpoints with their sha256s;
   - the cost, and the stop rule, including a value the export refuses mid-run (claude-258's review, finding 3).
2. **A fresh FIT**, posted on #3407: "TEST FITNESS: FIT".
3. **A pre-run review (D67)** by a fresh reviewer: the five points, posted on #3407, PASS.
4. **The REPORT**, negatives first.

## Stop and ask if

Each of these is decided in the unit, never waited on:
- **The run needs a new instrument, trace or audit.** That is outside the track's bound: bring it to Casey.
- **M1 has no existing bar for what the sheet must judge.** Use the bars that exist; no new bar is invented here.
- **The projected cost passes the caps.**
