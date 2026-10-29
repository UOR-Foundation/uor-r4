# Prep sheet: which part of the D11 forward carries its gap from float? (d11-exporter-float-trunk, claude-180), 2026-10-29

Lab: claude. Item: `d11-exporter-float-trunk` (#2820). ORDER: `docs/loop/tasks/d11-exporter-float-trunk/claude-180/ORDER.md`. This is R6's second half in `docs/model/pipeline-map.md`, under the owner direction of 29 Oct, "fix the exporter".

TEST FITNESS: PENDING

## 1. Question

**What claude-136 found.** Every export FAIL is the D11 forward's own gap from the float trunk.
- F against D is outside its paired F_I on all six models, by 77 to 359 rows of net top-1.
- The export itself moves each by at most 8 rows (D against A).

**How the D11 forward D differs from the plain-float forward F** (`with_d11_mode`, `evaluate_dev_panel_d11`):
1. **The gate snap.** Both Hamilton gates are snapped to the 120 icosians, as `Hamilton_IcosianHard` does. F keeps them continuous.
2. **The transition snap.** The recurrence transition is snapped to the 120 icosians. F unit-normalizes it.
3. **The rounding.** The tables are rounded as exported (`round_tables_as_exported`), and the scored query and the writer table are at Q12.

**The question: does F − D come from the snaps, from the rounding, or from both?**

**Why it decides the next unit.** Since D80 and D81 (28 Oct), the served artifact may use the integer multiply and divide.
- So a served gate or transition no longer has to be an icosian root, whose products are shifts.
- It can be an integer quaternion. The products become plain integer Hamilton products, and the transition's unit normalization becomes an integer square root and divide.
- If the snaps carry the gap, the fix is that unsnapped D11 contract. If the rounding carries it, the fix is wider codes, which the multiply makes cheap.

## 2. Base artifacts and arms

**The models: eight saved models, all trained 1,024 steps, scored on the 14,287-row `ft/dev` panel.** No training and no export.

| Label | Seed | Model folder in the store | `model.safetensors` sha256 | Training receipt (`--train-receipt`) |
|---|---|---|---|---|
| plain-c126 | 1 | `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/model/` | `63ae5513ff2da425af564be9ad9b41587a1ba0493f31061c06127a6368aa8e92` | `…/d11gap/d11-gap.json` |
| plain-c126 | 2 | `…/d11gap/save/seed-2/model/` | `37aae39685a141fc3dbe2bb0d0be653d002aa739fae9c3b0fcf55f4f06e2e487` | the same |
| d11-c130 | 1 | `results/claude/d11-fbs-d11-train-2026-10-26/d11t/save/seed-1/model/` | `55f5d6dce5ddb0121ab0a229f4ab62b0947afb61d52a7498879e7201c0843333` | `…/d11t/train.json` |
| d11-c130 | 2 | `…/d11t/save/seed-2/model/` | `f3c1b2064e885b90eef30b96404c4d17d86179dbaeba3eae37001f0e6316221c` | the same |
| dual-c131 | 1 | `results/claude/d11-fbs-dual-train-2026-10-26/dual/save/seed-1/model/` | `6861db0fb4a8773cb00ac0b50839c6007783b9de1ebf36d948ab38bf99d93a72` | `…/dual/train.json` |
| dual-c131 | 2 | `…/dual/save/seed-2/model/` | `a78ce0b942a906e3460de45f1e07db84b677d99beb09613b9373c9401dfac1cf` | the same |
| bfixed-c179 | 1 | `results/claude/d11-fbs-fixed-order-2026-10-29/fix-arms/n1/save/seed-1/model/` | `e66c5ca40dd943d386f66ab48b2aa074effef95fcbb3a3b7abda9be84e828ba1` | `…/fix-arms/n1-train.json` |
| bfixed-c179 | 2 | `…/fix-arms/n1/save/seed-2/model/` | `3fb45e442cc187344afe41063634be95f2e04080a7b7b2d5f4e6eb452497bd5b` | the same |

**The training receipts, pinned in full and checked by the job:**
- `d11-gap.json`: `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`. It is also the Stage 4b receipt (`--stage4b`) for every model, the source of the noise band.
- `d11t/train.json`: `9576720f4d66fe4c4d01baa616135ef227ec36af64da613fb51bf965e19a8d5f`.
- `dual/train.json`: `5a290fbe47d8f10495b6d89fa8384110061902001ad9ebdf6b365935e220d9bc`.
- `fix-arms/n1-train.json`: `79a6989761559f11575d4ad415b9e84761a89f55577807ec9e4684fc4f3a62aa`.

**What the receipts store:**
- Each receipt's `seeds[].loss_float` is `panel_loss(…, false)` at step 1,024. That is the same computation the mode repeats. Every receipt checked a reload against it exactly when its model was saved (`run_d11_export_gap` and `run_d11_train_export`).
- All four state the same noise band.

**Beside the models:**
- Each model's `preds-float.json` sits beside its `model/` folder. plain-c126 also has its Stage 4b `preds-d11.json`.
- Every file above is listed in its folder's `SHA256SUMS`.
- **What the eight are:**
  - claude-126 trained plain;
  - claude-130 trained in the D11 mode;
  - claude-131 trained dual;
  - claude-179's B_fixed trained dual with the fixed-order backward. It is the only one whose training repeats byte for byte.

**The arms.** Each model is scored as F, then as six D11 variants (`ABLATION_ARMS` in `spiralcore_fbs_sheet.rs`), in this order:

| Arm | Gate snap | Transition snap | Rounding | Role |
|---|---|---|---|---|
| `nothing` | off | off | off | instrument: it must be F (`I-None`) |
| `snaps_only` | on | on | off | the snaps alone |
| `d11` | on | on | on | D itself (`I-D-Same`) |
| `gates_continuous` | off | on | on | reading: D without the gate snap |
| `transition_continuous` | on | off | on | reading: D without the transition snap |
| `rounding_only` | off | off | on | the rounding alone |

- "Off" for a snap means F's own computation: the continuous gate (`Hamilton_Active`), or the unit-normalized transition. "Off" for the rounding means the tables as trained and the float writer.
- Rounding changes the model in place, so every unrounded arm runs first. A host test pins this order.
- Each arm's loss is the panel's `MarginHinge(rivals = 8)` loss under the same settings.

## 3. Bar and decision table

**The bar.** An arm X is **inside** when it passes the export gap's own bar against F (Stage 4b). Both halves must hold:
- **Top-1:** |b − c| ≤ F_I = max(24, ⌈2√(b + c)⌉). Here b counts the rows F hits and X misses, and c the reverse.
- **Loss:** |L_X − L_F| ≤ Δ = `0.045679637781173676`, Stage 4b's noise band.

**Carrying D's gap.** A part (`snaps_only` or `rounding_only`) **carries** D's gap when it is outside the bar on the half where D is, in D's direction.
- Where D's top-1 is outside, the part's top-1 must be outside, with a net of D's sign. D's net is positive when F is ahead, and negative when D is (d11-c130 seed 2 reads −340).
- Where only D's loss is outside, the part's loss must be outside, on D's side of F.

**Making up D's gap.** Two parts that are each inside **make up** D's gap when their values, taken in D's direction, sum to at least D's excess over its noise:
- on top-1, sign(net_D) · (net_snaps + net_rounding) ≥ |net_D| − F_I(D);
- on the loss half, the same with Δ in place of F_I(D).

**Each model's split** (`classify_snap_split`, in order):
1. **Not counted:** an instrument check (section 4) failed.
2. **No gap:** `d11` is inside.
3. **Both:** each part carries D's gap.
4. **Snap-carried:** the snaps carry it, and the rounding alone is inside.
5. **Rounding-carried:** the rounding carries it, and the snaps alone are inside.
6. **Both:** each part is inside, and together they make up D's gap. Each part is below the bar, and neither alone accounts for it.
7. **Interaction:** anything else. That is, the parts fall short of D's gap, or one part is outside against D's direction.

On plain-c126 seed 1 (D 77 rows against F_I 60), snaps at 45 and rounding at 32, each inside, read Both; at 8 and 8 they read Interaction. A host test pins these cases.

**The item's reading** is taken over the counted models whose split is not "no gap". The first row that holds applies:

| Reading | When | What it decides |
|---|---|---|
| **No reading** | fewer than 6 of the 8 models are counted | An instrument repair, then this run again. |
| **No gap** | no counted model has a gap | D is inside on every model, which contradicts claude-136. Investigate the instrument before reading anything. |
| **Snaps** | every one reads Snap-carried | **Next unit: the unsnapped D11 contract.** The exporter writes the gates and the transition as integer quaternions, and the integer runtime computes them, with the multiply and divide D80 and D81 allow. The unit includes finite CPU controls and the GPU evaluator, then the export gap on the plain recipe. Which snap goes first comes from `gates_continuous` and `transition_continuous`: one that is inside, or else the one with the smaller \|net\|. |
| **Rounding** | every one reads Rounding-carried | **Next unit: wider codes** for the query and the tables, with the snaps kept, then the export gap. |
| **Interaction** | more than half read Interaction | No single part is the fix. **Next unit: localize per layer** on those models before any contract change. |
| **Both** | otherwise | **Next unit: both changes.** Its sheet runs each change alone and the two together. Any model that reads Interaction is named there and localized as a reading. |

A Snaps or Rounding reading names the change for the next unit. It is not a result about the snaps or the rounding as mechanisms, and nothing here closes a mechanism (§4.1).

**Readings, with no bar:**
- each arm's top-1 and loss;
- b, c, net and F_I against F, with the rows that differ;
- each arm against `d11`: net, and the rows that differ;
- the 2×2 of gate and transition snaps on the rounded tables.

## 4. Instrument checks (each must be green before a model counts)

1. **`I-GPU-Tests`.** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on the pod. That includes the new `gpu_the_ablation_ends_are_d_and_f_and_each_snap_moves_the_queries`:
   - the all-snaps rounded arm is `evaluate_dev_panel_d11` row for row;
   - the empty arm is the float panel;
   - turning either snap off changes the D11 queries.
   A failure stops the run.
2. **`I-Fetch`.** Every fetched file verifies against its folder's `SHA256SUMS`, and the DEV data against its pins (section 5).
3. **`I-Inputs`.** The mode checks each `model.safetensors` against `--model-sha256`.
4. **`I-Save-Load`.** F reproduces `preds-float.json` row for row, and its loss is within 1e-6 of the training receipt's loss for that seed.
5. **`I-None`.** The `nothing` arm is F row for row, with its loss within 1e-6. This shows that "off" is the float forward.
6. **`I-D-Same`.** The `d11` arm is `evaluate_dev_panel_d11`'s panel row for row, with its loss within 1e-6 of `panel_loss_d11`'s. So this D is the D of claude-132 to claude-136.
7. **`I-D-Stored`, for plain-c126 only.** The `d11` arm equals Stage 4b's stored `preds-d11.json`.
8. **`I-D-Recorded`, for the six older models.** The `d11` arm's top-1 equals claude-136's D column: 1,911, 2,290, 3,666, 1,257, 3,486 and 2,903. The job passes each value with `--expect-d11-top1`, and the mode counts the check in `instruments_pass`, so a mismatch leaves that model not counted.
9. **`I-DEV-Denominator`.** F covers 14,287 rows.
10. **`I-Negative`.** plain-c126 seed 1 is run once more with a wrong `--model-sha256`. It must give `instruments_pass: false` and the split `not_counted`.
11. **`I-Finite`.** Every step exits 0, and a non-finite loss is an error in the mode. Each step's stdout, stderr and exit code are saved.

## 5. Data split

- DEV only: `w2-inputs/2026-10-25/driver-data/ft/dev/` — `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`. That is 428 windows and 14,287 scored rows.
- No training data is read, and neither is `fresh_v2`.
- All eight models were trained on `mw-balp/train` from step 0. The DEV panel is the one their receipts were scored on.

## 6. Steps

1. `scripts/pod/uor-pod up --lab claude --session claude-180 --count 1 --gpu 5090 --hours 1.5 --ref <pushed head> --card <#2820>`.
2. Build `spiralcore-fbs-sheet` (`-p uor-r4-float-train --features cuda`) with nvcc at the pushed head, and copy it out with its sha256.
3. `I-GPU-Tests`.
4. Fetch the section 2 files, each folder's `SHA256SUMS` and the DEV data (`I-Fetch`).
5. For each of the eight models, run the following, adding `--stored-d11-preds …/preds-d11.json` for plain-c126:

   `spiralcore-fbs-sheet --mode d11-snap-ablation --data-root <driver-data> --model-dir <…/model> --model-sha256 <pin> --stored-preds <…/preds-float.json> --stage4b <d11-gap.json> --train-receipt <receipt> --seed <s> --label <label> --out ablation-<label>-seed-<s>.json`
6. Run `I-Negative`.
7. A `jq` summary over the receipts (`summary.jq`, shipped with the job) gives each split and the item's reading. A model whose step failed has no receipt, so it is not counted.
8. Seal the run folder to `results/claude/d11-snap-ablation-2026-10-29/` with a `SHA256SUMS`, read it back, then `release` and `down`.

## 7. Minimum test power

- **Eight models:** four configurations, two seeds each.
- **14,287 frozen DEV rows per model**, with a paired test for every arm.
- **The forwards are deterministic.** claude-132 found D repeatable, and `I-D-Same` checks it again. So one pass decides each arm.
- **Most gaps are far outside, but not all.** claude-136's six read 77 to 359 rows of net against F_I of 58 to 72.
  - On five of them D is outside by 155 to 287 rows, so a part that carries most of the gap lands well outside.
  - **plain-c126 seed 1 is near the threshold,** outside by only 17 rows (77 against 60). There, both parts can sit inside the bar. The "make up" rule reads that case as Both, not Interaction.
  - **The two bfixed-c179 models have no recorded D.** Their gap is measured here for the first time, so either could read No gap, or be near the threshold.
- **No single model decides the item.** The item reading needs every gap model for Snaps or Rounding, and more than half for Interaction.

## 8. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.6 to $1.1/h, on a 1.5 h lease;
- about 3 minutes of build and 2 minutes of GPU tests;
- about 40 MB of models and receipts to fetch;
- 9 runs, each with 8 panels and 8 losses (F, the six arms, and the second D11 panel for `I-D-Same`). claude-135 measured about 54 s for five D11 panels, so each run should take 2 to 3 minutes.
- **In all, about 45 minutes, under $1.**

**Stop rule:**
- **The whole job stops,** and saves, on a failed build, `I-GPU-Tests`, `I-Fetch`, receipt-pin check, `I-Negative` or summary step.
- **Each model's own run is not fatal.** A nonzero exit there (a receipt it cannot read, say) writes no receipt, and that model is not counted. A failed instrument check inside a model's run leaves that model not counted too. The others still count, within the "No reading" row.
- Classify every failure as instrument or mechanism before calling it anything.
- Renew the lease rather than cancel a healthy run.
- Save everything off the pod and read it back before `down`.

## 9. Non-author TEST FITNESS review

**Round 1 (fresh non-author reviewer, at `38c2f5189`): NOT FIT. All eight findings are fixed.**
1. **High: one model could read Interaction while its parts simply add up,** and that one model then decided the item. Fixed:
   - "Make up D's gap" now reads two in-band parts that sum to D's excess as Both.
   - The item reading needs more than half of the gap models for Interaction.
2. **High: §7 claimed no near-threshold calls.** That was wrong. plain-c126 seed 1 is 17 rows outside, and bfixed-c179 has no recorded D. Both are now named.
3. **Medium: I-D-Recorded was a job check outside `instruments_pass`.** The mode now takes `--expect-d11-top1` and counts the check itself.
4. **Medium: the stored-loss check was untried.** Each receipt's `seeds[].loss_float` is `panel_loss(…, false)`, and each was checked exactly against a reload at save time. All four state the same band (section 2).
5. **Medium: the stop rule conflicted with the per-model rule.** Each model's run is now non-fatal (section 8).
6. **Low: the receipt pins were truncated.** They are in full now, and the job checks them.
7. **Low: the cost counted 7 panels per run.** It is 8.
8. **Low: a part's sign was ignored.** A part now carries D's gap only in D's direction.

**What round 1 confirmed:**
- each arm computes what section 2 says;
- `nothing` is F and `d11` is D by construction;
- the transition snap is the only use of `snap_transition` in the forward;
- the rounding order is enforced;
- the pins match claude-132's table;
- I-Negative can fail;
- no softmax, no matmul, GPU-only.
