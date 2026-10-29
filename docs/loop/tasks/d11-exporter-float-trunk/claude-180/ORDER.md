# ORDER: d11-exporter-float-trunk, claude-180

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `bcf2f57f7`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit's step toward it** (owner direction 29 Oct, "fix the exporter"; R6's second half in `docs/model/pipeline-map.md`):
- claude-136 showed that every export FAIL is the D11 forward's own gap from the float trunk (F − D), not the export (D − A, at most 8 rows).
- The D11 forward differs from F in four ways: the Hamilton gates snapped to the 120 icosians, the recurrence transition snapped to them too, the tables rounded as exported, and the query and writer at Q12.
- No run has split F − D among them. claude-133's Q16 control tested only the residual stream.
- **This unit measures that split**, eval only, on the eight saved models: claude-132's six, and claude-179's two repeatable B_fixed models. It adds one mode, `d11-snap-ablation`, that turns each snap off in the D11 forward.
- The split names the fix. D80 and D81 allow integer multiply and divide, so a served gate or transition no longer has to be an icosian root: the product can be a plain integer Hamilton product.

## Cancel probe (run first)

- **Is F − D already split by part?** Read the notebook rows for claude-129 to claude-140 (`docs/research/notebook/m4-serving.md`).
- **Does the repeatable recipe already have a D panel?** Read claude-179's sealed `n1-train.json`.

**Result (29 Oct): go. Class: mechanism (it decides which part of the forward the fix changes).**
- **The notebook has no split.** Two rows mention F against D: claude-132 (row 29) and claude-136 (row 24). Both give the whole F − D only: 77, 353, 275, −340, 359 and 225 rows of net top-1. claude-133 tested the residual stream's rounding alone (D16 against D, 8 to 18 rows on five models).
- **claude-179 left no D panel.** `n1-train.json` holds the float panel and the export-mode panel E for each seed, and nothing else. On seed 1, float top-1 is 1,475 and E is 1,733; on seed 2 they are 1,313 and 1,610.
  - E snaps the gate after Q12 rounding. It does not snap the transition or round the tables, so it is not D.
  - No `preds-d11.json` was saved.
- **B_fixed's model pins** come from claude-179's sealed `model-sha256.out`. `model.safetensors` is `e66c5ca4…ba1` for seed 1 and `3fb45e44…bd5b` for seed 2.

## Files in scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`: a D11 mode with each snap selectable, for evaluation only.
- `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs` and `crates/uor-r4-float-train/src/bin/spiralcore-fbs-sheet.rs`: the `d11-snap-ablation` mode.
- `docs/harness/prep/d11-exporter-float-trunk-snap-ablation-2026-10-29.md`
- `docs/loop/tasks/d11-exporter-float-trunk/claude-180/**`
- `docs/research/notebook/m4-serving.md`

## Done when

- The mode is merged with host tests, and a GPU test showing that the all-snaps setting reproduces `evaluate_dev_panel_d11` row for row.
- The sheet is FIT, and the D67 pre-run review passed.
- One RTX 5090 run has scored all eight models, with its results sealed in the store and read back.
- The REPORT gives each model's split, its outcome under the sheet's table, and the next unit that outcome names.
- One notebook row records it.
- The item is not closed by this unit.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- an instrument check fails: classify it, repair it, and run again;
- the all-snaps setting does not reproduce D: the mode is wrong, so stop the run;
- a change would touch the serving runtime, the exporter or training: that is the next unit's work.
