# Prep sheet: which change alone brings the D11 forward inside its gap from float? (d11-exporter-float-trunk, claude-180), 2026-10-29

Lab: claude. Item: `d11-exporter-float-trunk` (#2820). ORDER: `docs/loop/tasks/d11-exporter-float-trunk/claude-180/ORDER.md`. This is R6's second half in `docs/model/pipeline-map.md`, under the owner direction of 29 Oct, "fix the exporter".

TEST FITNESS: FIT

## 1. Question

**What claude-136 found.** Every export FAIL is the D11 forward's own gap from the float trunk.
- F against D is outside its paired F_I on all six models, by 77 to 359 rows of net top-1.
- The export itself moves each by at most 8 rows (D against A).

**How the D11 forward D differs from the plain-float forward F** (`with_d11_mode`, `evaluate_dev_panel_d11`):
1. **The gate snap.** Both Hamilton gates are snapped to the 120 icosians, as `Hamilton_IcosianHard` does. F keeps them continuous.
2. **The transition snap.** The recurrence transition is snapped to the 120 icosians. F unit-normalizes it.
3. **The rounding.** The tables are rounded as exported (`round_tables_as_exported`), and the scored query and the writer table are at Q12.

**The question: which change alone brings the D11 forward inside the export gap's bar against F — turning the snaps off, turning the rounding off, either, or neither?**

**Each change has an arm that computes it exactly** (section 2):
- **Snaps off** leaves `rounding_only`. That is what an unsnapped D11 contract computes, in float, before export.
- **Rounding off** leaves `snaps_only`. That is the limit of ever-wider codes, so a real widening such as Q16 lies between D and it.
- So "is that arm inside the bar?" answers "would that change alone be enough?" without any attribution of shares.

**Why it decides the next unit.** Since D80 and D81 (28 Oct), the served artifact may use the integer multiply and divide.
- So a served gate or transition no longer has to be an icosian root, whose products are shifts.
- It can be an integer quaternion. The products become plain integer Hamilton products, and the transition's unit normalization becomes an integer square root and divide.
- Wider codes are cheap with the multiply too.

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
- **All eight models were trained and saved on RTX 5090 pods**, like the one this run leases.
  - claude-126 trained on `vast-57786001`; claude-130, claude-131 and claude-179 each on one RTX 5090 (their notebook rows).
  - claude-132 reproduced the six older models' float panels row for row on a later 5090. A different GPU could break `I-Save-Load`'s 1e-6 loss check, which would waste the run rather than mislead it.

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

The bar is symmetric, so an arm is outside whichever of F and the arm is ahead. Each change is judged by the full bar, both halves, on the arm it leaves.

**Each model's split** (`classify_snap_split`, in order):
1. **Not counted:** an instrument check (section 4) failed.
2. **No gap:** `d11` is inside, so there is nothing for a change to close.
3. **Snaps off suffices:** `rounding_only` is inside and `snaps_only` is not.
4. **Rounding off suffices:** `snaps_only` is inside and `rounding_only` is not.
5. **Either suffices:** both are inside.
6. **Neither suffices:** neither is inside.

A host test pins every combination.

**The item's reading** is taken over the counted models whose split is not "no gap" (the gap models). "Snaps off works" means every gap model reads Snaps off suffices or Either suffices; "rounding off works" is the same with Rounding off. The first row that holds applies:

| Reading | When | What it decides |
|---|---|---|
| **No reading** | fewer than 6 of the 8 models are counted | An instrument repair, then this run again. |
| **No gap** | no counted model has a gap | D is inside on every model, which contradicts claude-136. Investigate the instrument before reading anything. |
| **No reading** | 1 to 3 gap models | Too few models to name a change. The six older models' D top-1 is held by `I-D-Recorded`, so it fixes their nets at 77, 353, 275, −340, 359 and 225. So this row arises only if most of them are not counted. |
| **Snaps off** | snaps off works; and if rounding off works too, the snaps-off arm's \|net\| summed over the gap models is no larger than the rounding-off arm's | **Next unit: the unsnapped D11 contract.** The exporter writes the gates and the transition as integer quaternions, and the integer runtime computes them, with the multiply and divide D80 and D81 allow. The unit includes finite CPU controls and the GPU evaluator, then the export gap on the plain recipe. `gates_continuous` and `transition_continuous` say whether one snap could stay. `rounding_only` keeps the gates and transition in float, so their own integer rounding (and the transition's integer square root and divide) is that unit's export-gap question, not this run's. |
| **Rounding off** | rounding off works, and the row above does not hold | **Next unit: wider codes** for the query and the tables, with the snaps kept, then the export gap. The arm is the limit of wide codes, so that sheet sets the width from a ladder (Q16 first). |
| **Both off** | neither change works on every gap model | **Next unit: both changes.** Its sheet runs each change alone and the two together. The models where neither alone suffices are named there. |

The tie-break (when both changes work everywhere) is pre-registered here: the change whose arm sits closer to F over the gap models goes first. On a tie the snaps go first.

The two guard rows, "No gap" and "1 to 3 gap models", cannot be reached when 6 models are counted. At least 4 of the six older models are then among them, and `I-D-Recorded` and `I-Save-Load` fix their nets outside F_I. They stay as guards against a broken instrument.

A reading names the change for the next unit. It is not a result about the snaps or the rounding as mechanisms, and nothing here closes a mechanism (§4.1).

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
11. **`I-Finite`.**
    - Every job-level step exits 0.
    - A nonzero exit in a model's own run leaves that model uncounted (section 8).
    - A non-finite loss is an error in the mode.
    - Each step's stdout, stderr and exit code are saved.

## 5. Data split

- DEV only: `w2-inputs/2026-10-25/driver-data/ft/dev/` — `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`. That is 428 windows and 14,287 scored rows.
- No training data is read, and neither is `fresh_v2`.
- All eight models were trained on `mw-balp/train` from step 0. The DEV panel is the one their receipts were scored on.

## 6. Steps

**The job and its summary program** are local files, shipped to the pod's `/workspace/abl-job/`.
- **`summary.jq` is pinned** at `200425f24393c20f9c6d4f4d2bafb97808e38f61586c863cacd41e152012cf16`. The job checks it before use.
- **`job.sh` is pinned as a template** at `d82e86c27dc75d78d360de3ad2dc689f65bc2d6bda585b788f53a55e94e86f5d`. The launched copy differs only in its `SHA=` line, which takes the pushed head.
- **The job copies both into the run folder,** so the seal holds the exact programs that produced the reading.
- **The job refuses to start if its run folder is not empty,** so a retry cannot count a stale receipt.
  - The refusal writes `/workspace/abl-refused`, outside the run folder.
  - The watcher ends on that marker, or on `/workspace/abl/EXIT`, whichever appears first.

1. `scripts/pod/uor-pod up --lab claude --session claude-180 --count 1 --gpu 5090 --hours 1.5 --ref <pushed head> --card <#2820>`.
2. Build `spiralcore-fbs-sheet` (`-p uor-r4-float-train --features cuda`) with nvcc at the pushed head, and copy it out with its sha256.
3. `I-GPU-Tests`.
4. Fetch the section 2 files, each folder's `SHA256SUMS` and the DEV data (`I-Fetch`).
5. For each of the eight models, run the following, adding `--stored-d11-preds …/preds-d11.json` for plain-c126:

   `spiralcore-fbs-sheet --mode d11-snap-ablation --data-root <driver-data> --model-dir <…/model> --model-sha256 <pin> --stored-preds <…/preds-float.json> --stage4b <d11-gap.json> --train-receipt <receipt> --seed <s> --label <label> --out ablation-<label>-seed-<s>.json`

   For the six older models, also add `--expect-d11-top1 <claude-136's D>`. Before the runs, the job checks the four receipts' full pins, and checks that each receipt's `seeds[].i_save_load` is true for both seeds.
6. Run `I-Negative`.
7. A `jq` summary over the receipts (`summary.jq`, shipped with the job) gives each split and the item's reading. A model whose step failed has no receipt, so it is not counted.
8. Seal the run folder to `results/claude/d11-snap-ablation-2026-10-29/` with a `SHA256SUMS`, read it back, then `release` and `down`.

## 7. Minimum test power

- **Eight models:** four configurations, two seeds each.
- **14,287 frozen DEV rows per model**, with a paired test for every arm.
- **The forwards are deterministic.** claude-132 found D repeatable, and `I-D-Same` checks it again. So one pass decides each arm.
- **Most gaps are far outside, but not all.** claude-136's six read 77 to 359 rows of net against F_I of 58 to 72.
  - On five of them D is outside by 155 to 287 rows.
  - **plain-c126 seed 1 is near the threshold,** outside by only 17 rows (77 against 60). There, both arms can be inside, which reads Either suffices. That is a true reading, not a fault: on this model, each change alone is enough.
  - **The two bfixed-c179 models have no recorded D.** Their gap is measured here for the first time, so either could read No gap.
- **How many models a reading needs.**
  - A Snaps off or Rounding off reading needs that change to work on every gap model, and at least 4 gap models.
  - A single model can still decide between those two, where it is the only one on which a change fails. That is the intended strictness, since the next unit's change must close the gap on every model.

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

**D67 pre-run review (fresh reviewers): GO in round 2, at `2e98ce155`.**
- **Round 1, at `cc1696a83`: NO-GO.** It checked the job, the store inputs, the summary program, the FIT record and the caps. Its one fault was that the job's early refusal left no marker, so a watcher could wait out the lease. It ran out of budget before checking the chain.
- **Round 2 checked the chain:**
  - pipeline-map gap 5 and R6;
  - the item's done-when;
  - claude-136's bound.
  It found that measuring first is the right order, and that every row names a different next unit, with no mechanism claim. It also confirmed the refusal fix and the job pin.
- **Its three low findings were done before launch:**
  - an earlier launch's refusal marker is cleared (job re-pinned);
  - CHARTER §3.6 reads as the sheet says (D80 integer multiply, D81 integer divide);
  - all six `--expect-d11-top1` values match claude-136's table.

**Round 3 (fresh non-author reviewer, at `4be16c024`): FIT.** It found nothing blocking or high, and ran the five relevant CPU tests at that commit (5 passed). Its findings are folded in:
1. **Medium:** the job and summary program are now pinned, checked and sealed (section 6).
2. **Medium:** a retry into a used run folder is refused (section 6).
3. **Low:** I-Finite now separates job-level steps from a model's own run (section 4).
4. **Low:** the two unreachable guard rows are named as guards (section 3).
5. **Low:** the Snaps-off row says what `rounding_only` leaves to the next unit (section 3).
6. **Low:** the training GPUs are recorded (section 2).

**Round 1 (fresh non-author reviewer, at `38c2f5189`): NOT FIT.** Its fixes for findings 1 and 8 were replaced in round 2 by the current rule. The rest stand.
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

**Round 2 (fresh non-author reviewer, at `ad6f65485`): NOT FIT. All six findings are fixed.**
1. **High: the "make up" rule did not match the sheet's own words.** For example, snaps 0 with rounding 17 read Both.
   - The fault was the framing: splitting shares asked a question no decision needs.
   - The split now asks only which change alone brings the forward inside the full bar (§3). `rounding_only` and `snaps_only` are exactly the forwards those changes leave.
   - So no sign, sum or share rule remains.
2. **Medium: "inside" mixed the halves.** It is gone with the same change. Each arm is judged by both halves of the bar, as D is.
3. **Medium: one model could set the next unit.**
   - A reading now needs at least 4 gap models.
   - §7 says where a single model still decides, and why that strictness is intended.
4. **Low: claude-179's receipt writer was unconfirmed.** `n1-train.json` was written by `--mode d11-train-export` (claude-179's `arms.sh`, line 49), which is `run_d11_train_export`. All four receipts read `i_save_load: true` on both seeds, and the job now checks this.
5. **Low: the tolerance.** `reloaded_loss_matches` is `(actual − expected).abs() <= 1e-6` (`spiralcore_fbs_sheet.rs:1907`). That is what §4 says.
6. **Low: a job step that could not fail** (`receipts-written`) was removed.

**What round 1 confirmed:**
- each arm computes what section 2 says;
- `nothing` is F and `d11` is D by construction;
- the transition snap is the only use of `snap_transition` in the forward;
- the rounding order is enforced;
- the pins match claude-132's table;
- I-Negative can fail;
- no softmax, no matmul, GPU-only.

## 10. Attempt 2: the six older models with their own kernel (an instrument repair)

**What attempt 1 showed** (pod `vast-58216485`, binary `075d88e2…`, at `9a6002c9b`).
- The GPU library tests passed: 163 passed and 6 ignored, the new ablation test among them.
- The two bfixed-c179 models ran.
- **The six older models were refused at load.** `StackModel::load` reported "native projection compiler/operator/FBS identity is missing or stale".
  - Each FBS plan's fingerprint includes `kernel_sha256`, the sha256 of `spiralcore_fbs.cu`, which claude-179 changed (`517bffcc…` → `3fbfc796…`).
  - So every model saved before #3367 is bound to the old kernel source.
- The non-fatal per-model rule worked: those six wrote no receipt.
- **`I-Negative` was also refused at load,** so the job stopped there, before its summary. It never reached the pin it was meant to test.

**Class: instrument.** The run, not the models, was wrong. It had no binary that may load them. claude-179 changed only the backward kernels, and this run evaluates forwards only. The models are the ones sealed by their own runs, with their pins unchanged.

**The repair, `job2.sh` (sha256 `9582edd7af8f02badf7b84081c039ef1e0eaacc975094f2618e9c9cda9a5228e`):** a second binary, B_old, from the same head with claude-179's `fix.patch` reversed. That is how claude-179 built its B_base.
- `fix.patch` (sha256 `a2728e80…9d55`) is checked first. Then:
  - B_old's `crates/uor-r4-float-train/src` differs from the head's in exactly three files, with no `.orig` or `.rej` leftovers;
  - its `spiralcore_fbs.cu` is `517bffcc…0b0d`, the old pin;
  - the two D11 GPU tests (the ablation's ends, and the head on the D11 queries) pass on it.
- **B_old scores the six older models** with the same commands and checks as section 6. It also runs `I-Negative`, on a model it loads.
- **Attempt 1's folder is kept as it is** (L-002). Attempt 2 writes `/workspace/abl2`, re-checks attempt 1's verified inputs against attempt 1's lists, and refuses a non-empty folder.
- **The summary** runs the same pinned `summary.jq` over all eight receipts: attempt 1's two, and attempt 2's six.
- Sections 3 and 4 are unchanged.

**One reading was seen before this repair:** the two bfixed-c179 receipts. The repair changes no bar, no arm and no reading rule, and does not touch those two models. It only gives the other six a binary that can load them.

**The repair's fresh pre-run review: GO.**
- **The patch reverses only the backward.** Every hunk of `fix.patch` sits in the backward kernels or their launchers, and no forward line changes.
- **The stored forward is pinned twice.** `I-Save-Load` and `I-D-Recorded` pin each model's forward against what it stored.
- **Section 10 overclaims nothing.**
- **Its four findings were folded in before launch:**
  - the summary runs before `I-Negative`;
  - attempt 1's two receipts are checked against its sealed `SHA256SUMS`;
  - the three-file check now names its folder and also refuses `.orig` and `.rej` leftovers;
  - the job stops if the first older model is still refused at load.
