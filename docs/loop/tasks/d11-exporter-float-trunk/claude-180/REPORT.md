# REPORT: d11-exporter-float-trunk, claude-180 (R6: which change alone closes the D11 forward's gap)

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-snap-ablation-2026-10-29.md](../../../../harness/prep/d11-exporter-float-trunk-snap-ablation-2026-10-29.md). References #2820. The item is not done.

**Speed.** Not a serving result. On the pod, each model's 8 panels and 8 losses over 14,287 DEV rows took about 64 s.

## What did not work, first

- **The registered reading is Both off.** Neither change alone brings every model inside the export bar.
  - Turning the snaps off is enough on 2 of 8 models, and turning the rounding off on 1. Neither alone is enough on the other 5.
  - This is a reading for the next unit's design, not a mechanism result (§4.1).
- **Turning the snaps off misses the bar on six models, three of them narrowly.** The narrow three have |net| of 31, 56 and 52 against F_I of 24, 29 and 24, missing by 7, 27 and 28 rows.
  - When the forward is nearly F's, few rows differ, so F_I is at or near its floor of 24.
  - The bar was not moved.
- **Attempt 1 could not load six of the eight models.**
  - Each FBS plan's fingerprint includes the sha256 of `spiralcore_fbs.cu`, which claude-179 changed. So the head refuses every checkpoint saved before #3367 ("native projection compiler/operator/FBS identity is missing or stale").
  - claude-179 did not flag that consequence. It applies to any lab that loads those checkpoints at `main`.
  - Attempt 1's `I-Negative` used one of those models, so it was refused at load too, and never tested the pin.
  - **The repair** was attempt 3's binary B_old: the same head with claude-179's `fix.patch` reversed. That gives each model the kernel it was saved with. It was reviewed fresh, GO (sheet §10).
- **Attempt 2 stopped on my own leftovers check, before any build.** The check flagged the vendored `Cargo.toml.orig` files the head already holds. Attempt 3 compares against the head.
- **claude-179 left a red test on main, and no CI job saw it.**
  - claude-179 changed `spiralcore_fbs.cu` without moving its pin, so `the_pins_are_the_sheets_and_still_match_the_sources` has failed since #3367.
  - No workflow runs `uor-r4-float-train`'s tests. I had run only that crate's dependency test in claude-179.
  - This unit moves the pin and runs the whole crate. The missing CI job is logged in FRICTION.md (`.github/**` is an owner path).
- **The sheet took three FIT rounds,** and the decisive fault was my framing.
  - Rounds 1 and 2 split F − D into shares carried by each part, and each round found a mislabelling.
  - The fix asked only what the next unit needs: is the forward with the snaps off (`rounding_only`), or with the rounding off (`snaps_only`), inside the bar?
- **The D67 review took two rounds.** Round 1 ran out of budget before reaching the chain; round 2 gave GO.

## What changed

- **`hamilton_sheet.rs`:** `with_d11_snaps` and `D11Snaps` run the D11 forward with the gate snap and the transition snap chosen apart. `with_d11_mode` is now its all-snaps case. `evaluate_dev_panel_d11_snaps` scores a panel under them.
- **`spiralcore_fbs_sheet.rs`:**
  - `run_d11_snap_ablation` scores F and six variants (`ABLATION_ARMS`) per model, and `classify_snap_split` gives each model's split.
  - `seed_float_loss` reads a training receipt's stored float loss.
  - The `spiralcore_fbs.cu` pin moves to claude-179's kernel.
  - Host tests: the hook's set and restore, every split combination, the arm order, and the receipt reader.
  - A GPU test: the ablation's ends are D and F, and each snap moves the D11 queries.
- **`bin/spiralcore-fbs-sheet.rs`:** the `d11-snap-ablation` mode, with `--train-receipt` and `--expect-d11-top1`.
- **Docs:** the prep sheet, this ORDER and REPORT, and one notebook row.

## Measurements

**The run.** One RTX 5090 on pod `vast-58216485`: driver 580.178.04, CUDA 12.8, $0.563/h.
- The pod was created at about 07:28Z (its bootstrap began at 07:30:24Z, after 119 s waiting for SSH).
- Attempt 3 ended at 07:53:27Z.
- The pod was deleted after the seal was read back. No local log records the delete time.
- So it was up for about 30 to 35 minutes, about $0.3.
- **Binaries:** `075d88e2…` (the head, `9a6002c9b`) for the two bfixed-c179 models. `7d2705ab…` (B_old: the head plus `fix.patch` reversed, exactly three files apart, with `spiralcore_fbs.cu` at the old pin `517bffcc…`) for the six older models.
- **Sealed** to `caseyallard/uor-r4-store:results/claude/d11-snap-ablation-2026-10-29/`. That is 379 files across all three attempts. The `SHA256SUMS` sha256 is `4e41463b3ce81339efd4a7e35b608a3d5cabea627535c45d5241b32668c7fdb2`, and it read back with every file verifying.

**Instruments. All green; all eight models are counted.**
- `I-GPU-Tests`: the head had 163 passed and 6 ignored, including the new ablation test. On B_old, the two D11 GPU tests passed.
- `I-Fetch`, the receipt pins and each receipt's `i_save_load` all held.
- `I-Save-Load`, `I-None` and `I-D-Same` held on all 8 models.
  - `I-D-Same` runs the same code path as the all-snaps arm a second time. So it shows the D panel is deterministic, not that it is independently D.
  - The independent ties to earlier units' D are the next two checks.
- `I-D-Stored` held on both plain-c126 seeds: D matched the predictions Stage 4b stored.
- `I-D-Recorded` held on all 6 older models: D reproduced claude-136's top-1 exactly.
- `I-Negative` (attempt 3): a wrong pin gave `i_inputs: false` and `not_counted`.

**Each arm against F: net top-1, with F_I in brackets.** Net is F's top-1 minus the arm's, so it is positive when F is ahead. **Bold** marks an arm inside the top-1 bar. Every arm's loss is inside the band, Δ = 0.0457; the largest |ΔL| is 0.031.

| Model | F top-1 | D | Snaps off | Rounding off | Gate snap off | Transition snap off | Split |
|---|---|---|---|---|---|---|---|
| plain-c126 s1 | 1,988 | 77 (60) | −151 (56) | 133 (48) | −164 (58) | **7 (56)** | neither |
| plain-c126 s2 | 2,643 | 353 (69) | −31 (24) | 152 (66) | **−23 (34)** | 368 (66) | neither |
| d11-c130 s1 | 3,941 | 275 (68) | 56 (29) | 120 (67) | **−14 (37)** | 333 (71) | neither |
| d11-c130 s2 | 917 | −340 (58) | −52 (24) | −271 (58) | −42 (32) | −605 (68) | neither |
| dual-c131 s1 | 3,845 | 359 (72) | **3 (24)** | 366 (73) | **16 (45)** | 402 (73) | snaps off |
| dual-c131 s2 | 3,128 | 225 (70) | **−9 (24)** | 234 (73) | −52 (45) | 103 (70) | snaps off |
| bfixed-c179 s1 | 1,475 | −230 (45) | −72 (27) | −54 (36) | −178 (36) | −132 (43) | neither |
| bfixed-c179 s2 | 1,313 | −259 (42) | −120 (32) | **−10 (31)** | −72 (31) | −191 (41) | rounding off |

**Readings, with no bar:**
- **Snaps off closes most of D's gap on 7 of 8 models.**
  - |net| falls 353 → 31, 275 → 56, 340 → 52, 359 → 3, 225 → 9, 230 → 72 and 259 → 120.
  - On plain-c126 seed 1 it overshoots instead, from 77 to −151, which leaves the forward ahead of F.
  - The rows that differ from F fall from D's 7,516–11,828 to 1,087–3,611 on five models.
  - They fall less on the two bfixed models, to 5,166 and 9,673, and on plain-c126 seed 1, to 7,737.
- **Rounding off leaves the six older models' |net| at 120–366, all outside F_I.**
  - It roughly halves D's on plain-c126 s2 (353 → 152) and d11-c130 s1 (275 → 120).
  - It falls by a fifth on d11-c130 s2 (340 → 271), and changes little on dual-c131 s1 and s2 (359 → 366, 225 → 234).
  - It rises on plain-c126 s1 (77 → 133).
  - Their differing rows stay near D's.
  - On the two bfixed models it closes most of the gap (−54 and −10).
- **On six of eight models, the gate snap carries most of the gap the snaps make.**
  - With the gate snap off, |net| is 14 to 52 on five models, and three of those are inside (plain-c126 s2, d11-c130 s1, dual-c131 s1).
  - On bfixed-c179 s2, gate snap off goes from 259 to 72, below snaps off's 120, but stays outside F_I (31).
  - On bfixed-c179 s1, gate snap off goes only from 230 to 178. On plain-c126 s1 it overshoots to −164, while transition snap off brings that model inside (7).
  - With only the transition snap off, |net| stays at D's size or above on four models (368, 333, 605, 402).
  - It falls on dual-c131 s2 (225 → 103) and both bfixed models (230 → 132, 259 → 191), but stays outside F_I.
- **The tie-break did not apply.** The summed |net| over the eight models is 494 with the snaps off and 1,340 with the rounding off.
- **The repeatable recipe's first D panel** (bfixed-c179) has D ahead of F by 230 and 259 rows, outside F_I of 45 and 42. So its export gap, at the level of the D11 forward, fails on both seeds, in D's favour.

## Checks run at the head

- `cargo test --locked -p uor-r4-float-train --no-fail-fast` on the host: 104 library tests and the bin and integration tests pass.
- The stub-`nvcc` `cargo clippy --features cuda --all-targets`: no warnings in the changed code. The one hit in the file, `tensors_of`'s type complexity, is older.
- The pod's GPU tests, above.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The bar is the export gap's own, unchanged. The question was narrowed to the decision it serves, the next change to the exporter, and nothing is claimed about the snaps or the rounding as mechanisms.
- **solution-integrity.** The six refused models were given the kernel they were saved with; the identity check was not weakened. Each attempt's folder was kept and sealed.

## Still open

- **The next unit (the sheet's Both off row): both changes in the integer contract.** Its sheet runs each change alone and the two together.
  - **The exporter and the integer runtime serve the Hamilton gates unsnapped,** as integer quaternions with D80's multiply. The gate goes first, because it carries most of the snaps' gap on six of eight models. The transition follows, since plain-c126 s1 needs it: its unit normalization needs D81's divide and an integer square root.
  - **The query and the tables go to wider codes,** a ladder starting at Q16.
  - **Then the export gap of each arm.** Neither change alone is enough on plain-c126 (both seeds), d11-c130 (both seeds) and bfixed-c179 seed 1.
- **Loading an older checkpoint.** Every checkpoint saved before #3367 needs the pre-#3367 `spiralcore_fbs.cu` to load at `main`. B_old's recipe (sheet §10) is how.
- **A CI job for `uor-r4-float-train`'s tests** needs the owner (`.github/**`), and is logged in FRICTION.md.
- **The mechanism page** `docs/mechanisms/lut-serving.md` is out of this item's scope (claude-121).

## Plain summary

Earlier work found that the integer version of the model, the one a phone or laptop would run, picks different next words from the trained model, and that the difference comes from the integer version itself, not from the export. That version changes the trained model in two ways: it rounds some rotations to the nearest of 120 fixed ones, and it rounds its numbers to a fixed number of digits. This run tried each change alone on eight trained models. Removing the rounding to 120 rotations brings the integer version much closer to the trained model on seven of the eight, and on six of them the rotation in the gates is the main one. But neither change alone is enough on every model. The next step builds an integer version that keeps the gate rotations exact, now that integer multiplication is allowed, and uses more digits, then measures again.
