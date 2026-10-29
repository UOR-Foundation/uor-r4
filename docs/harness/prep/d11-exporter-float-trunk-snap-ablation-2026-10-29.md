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
| plain-c126 | 1 | `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/model/` | `63ae5513ff2da425af564be9ad9b41587a1ba0493f31061c06127a6368aa8e92` | `…/d11gap/d11-gap.json`, `88b6a5f9…1a8a` |
| plain-c126 | 2 | `…/d11gap/save/seed-2/model/` | `37aae39685a141fc3dbe2bb0d0be653d002aa739fae9c3b0fcf55f4f06e2e487` | the same |
| d11-c130 | 1 | `results/claude/d11-fbs-d11-train-2026-10-26/d11t/save/seed-1/model/` | `55f5d6dce5ddb0121ab0a229f4ab62b0947afb61d52a7498879e7201c0843333` | `…/d11t/train.json`, `9576720f…8d5f` |
| d11-c130 | 2 | `…/d11t/save/seed-2/model/` | `f3c1b2064e885b90eef30b96404c4d17d86179dbaeba3eae37001f0e6316221c` | the same |
| dual-c131 | 1 | `results/claude/d11-fbs-dual-train-2026-10-26/dual/save/seed-1/model/` | `6861db0fb4a8773cb00ac0b50839c6007783b9de1ebf36d948ab38bf99d93a72` | `…/dual/train.json`, `5a290fbe…d9bc` |
| dual-c131 | 2 | `…/dual/save/seed-2/model/` | `a78ce0b942a906e3460de45f1e07db84b677d99beb09613b9373c9401dfac1cf` | the same |
| bfixed-c179 | 1 | `results/claude/d11-fbs-fixed-order-2026-10-29/fix-arms/n1/save/seed-1/model/` | `e66c5ca40dd943d386f66ab48b2aa074effef95fcbb3a3b7abda9be84e828ba1` | `…/fix-arms/n1-train.json`, `79a69897…62aa` |
| bfixed-c179 | 2 | `…/fix-arms/n1/save/seed-2/model/` | `3fb45e442cc187344afe41063634be95f2e04080a7b7b2d5f4e6eb452497bd5b` | the same |

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

**Each model's split** (`classify_snap_split`, in order):
1. **Not counted:** an instrument check (section 4) failed.
2. **No gap:** `d11` is inside.
3. **Snap-carried:** `rounding_only` is inside and `snaps_only` is outside.
4. **Rounding-carried:** `snaps_only` is inside and `rounding_only` is outside.
5. **Both:** both are outside.
6. **Interaction:** both are inside, while `d11` is outside.

**The item's reading** is taken over the counted models whose split is not "no gap", applying the first row that holds:

| Reading | When | What it decides |
|---|---|---|
| **No reading** | fewer than 6 of the 8 models are counted | An instrument repair, then this run again. |
| **Interaction** | any model reads Interaction | No single part is the fix. Next: localize per layer on those models before any contract change. |
| **Snaps** | every model reads Snap-carried | Next unit: the unsnapped D11 contract. The exporter writes, and the integer runtime computes, the gates and the transition as integer quaternions, using the multiply and divide D80 and D81 allow. It comes with finite CPU controls, the GPU evaluator, and then the export gap on the plain recipe. `gates_continuous` and `transition_continuous` say which snap goes first: an arm that is inside, or else the smaller \|net\|. |
| **Rounding** | every model reads Rounding-carried | Next unit: wider codes for the query and the tables, with the snaps kept, and then the export gap. |
| **Both** | otherwise | Next unit: both changes. Its sheet arms each change alone and both together. |

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
8. **`I-D-Recorded`, for the six older models (a job check).** The `d11` arm's top-1 equals claude-136's D column: 1,911, 2,290, 3,666, 1,257, 3,486 and 2,903.
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
7. A `jq` summary over the eight receipts gives each split, the item's reading and `I-D-Recorded`.
8. Seal the run folder to `results/claude/d11-snap-ablation-2026-10-29/` with a `SHA256SUMS`, read it back, then `release` and `down`.

## 7. Minimum test power

- **Eight models:** four configurations, two seeds each.
- **14,287 frozen DEV rows per model**, with a paired test for every arm.
- **The forwards are deterministic.** claude-132 found D repeatable, and `I-D-Same` checks it again. So one pass decides each arm.
- **The gaps are large.** They are 77 to 359 rows of net, against F_I of 58 to 72. A part that carries most of the gap lands far outside, and a part that carries none of it lands near zero, so the split is not a near-threshold call on these models.

## 8. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.6 to $1.1/h, on a 1.5 h lease;
- about 3 minutes of build and 2 minutes of GPU tests;
- about 40 MB of models and receipts to fetch;
- 9 runs, each with 7 panels and 7 losses. claude-135 measured about 54 s for five D11 panels, so each run should take 2 to 3 minutes.
- **In all, about 45 minutes, under $1.**

**Stop rule:**
- Stop and save on a failed `I-GPU-Tests` or `I-Fetch`, or on any nonzero exit.
- A failed instrument check on one model leaves that model not counted. The others still count, within the "No reading" row.
- Classify every failure as instrument or mechanism before calling it anything.
- Renew the lease rather than cancel a healthy run.
- Save everything off the pod and read it back before `down`.

## 9. Non-author TEST FITNESS review

To be recorded here, with the D67 pre-run review.
