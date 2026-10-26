# ORDER: r-rank-objective-keeps-cells, antigravity-97

Lab: antigravity. Item: [r-rank-objective-keeps-cells](../../../../roadmap/items/r-rank-objective-keeps-cells.md). Base: `df9ac0f25`.

## Goal

The item's done-when reads:

> A rank objective that passes G-Occ (more than 16 of 256 cells, plus the D39 condition) and is evaluated against the count tables.

## Cancel probe (run first)

- **Probe executed (`2026-10-26`, classed `mechanism` — configuration negative on cell-only rank hinges without within-cell token discrimination; `mechanism_negative: false`):**
  1. Verified all upstream prerequisites for `r-rank-objective-keeps-cells` (`#2883`):
     - `m1-a8-full-dose-rerun` (`#2831`, PR `#3192`, `cb2b5748905dc77b122325101b62dd7c29fc1413`) is `done` (`Q1` `FrozenZero` `DEV b_3(Full) = 4,292 / 14,287`, `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV top-1 = 871 / 14,287`, `occupied_cells >= 192 / 256`, `G-Occ PASS`).
     - `m1-bvg-k-stage-t-training` (`#2836`, PR `#3239`, `025e6f4f0565855ba08135968466cbdd61b08b89`, HF archive `stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`, `grade.json` `cd66a41b77050c01511b40ff61cf07c86116bf05344566ab642d1a07213e917a`) is `done`: on both seeds (`1` and `2`), `Arm-A` (`T16_fz`, `NearestAddressMarginV1` + `AddressCellsA8` + `D40` + `BVG-K` `K = 16`) already satisfies both `G-Occ` (`occupied_cells = 256 / 256 > 16`, `>= max(32, ceil(0.5 * 256)) = 128`, `mean_admitted_tokens = 256 <= 512`, `max_cell_tokens = 16 <= 32`) and `G-Adm` (`DEV b_3(Full) = 5,009 / 14,287 >= 4,894`, `DEV b_3(P_succ) = 1,216 / 3,847 >= 1,106`), while `OuterL1V1` `DEV writer top-1 = 455 / 14,287` (`T16_rel = 442 / 14,287`).
     - `m1-within-cell-writer` (`#3267`, PR `#3285`, `stage-t-decomposition.json` `e9dd940c0f58a1c494bb5d8f38a485528b6693bcaa67b23df3f1d867bec76cc6`) proved on the exact `T16_fz` checkpoint that `4,208 / 5,009` (`84.01%`) of admitted DEV targets lose to a rival token **inside their own endpoint cell `target_cell`**, and `1,597 / 1,765` (`90.48%`) of missed `T_restr(A_W) = 2,140` hits lose within their own cell.
  2. Verified the exact algebraic and integer properties of `FixedGridRankHingeV1` (`R_grid`, `s[11] = 2`) and `OccupiedCellRankCrowdHingeV1` (`R_occ`, `s[11] = 3`) registered in `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`:
     - Under `R_grid` and `R_occ`, $r_{\text{grid}}(\text{row}) = 1 + \#\{c \in \{0,\dots,255\} : \text{cell\_precedes}(d(q, c), c, \text{ad}, c_{\text{target}})\}$ ranges over all 256 fixed-grid cells whether occupied or empty (`C-R7a`), $8\,C_{\text{crowd}}(\text{row})$ penalizes any cell with $n(c) > 24$ (`C-R7b`), and the move-time `G-Occ` guard refuses any endpoint move (`block 0` or `block B-1`) that drops $\text{occupied\_cells} < 128$ or raises $n(c_{\text{new}}) > 32$ (`C-R7c`). Thus `R_occ` and `R_grid` strictly eliminate the `K16-R` collapse (`20` and `19` occupied cells, $\bar{n}_R = 3,403.9$ and $3,480.7$) and provably hold `G-Occ`.
     - However, when all 256 cells are occupied with $n(c) = 16 \le 24$ (`AddressCellsA8`), $r_{\text{grid}}(\text{row}) \equiv r_{\text{occ}}(\text{row})$ and $C_{\text{crowd}}(\text{row}) \equiv 0$; on every non-endpoint move (`Combine`, `Relation`, or interior token blocks `1..B-1`), `occupied[c]` and `n(c)` are invariant; and on all `5,009` admitted DEV rows ($r_{\text{grid}}(\text{row}) \le K = 16$), $R_{\text{grid}}(\text{row}) = 0$, so `R_occ` and `R_grid` supply **zero within-cell token-ranking signal** (`C-R7d`).
  3. Under CHARTER §3.5 (*"Spend GPU only where a run can change a decision"*), §5.1, and D68, authored `docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/DECISION.md` to implement `FixedGridRankHingeV1` (`s[11]=2`) and `OccupiedCellRankCrowdHingeV1` (`s[11]=3`), qualify all finite controls (`C-R1..C-R6`, `C-R7a..C-R7d`), and grade `r-rank-objective-keeps-cells` (`#2883`) against `G-Occ` and the count tables (`U = 1,226`, `T2 = 6,300`, `T(c(x)) = 4,085`, `U_restr(A_W) = 1,127`, `T_sym_restr(A_W) = 1,909`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`) as **`FAIL_RANK_GAIN_AT_HIGH_OCC`** (`mechanism_negative: false`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/r-rank-objective-keeps-cells*`
- `docs/labs/r-rank-objective-keeps-cells/**`
- `docs/loop/tasks/r-rank-objective-keeps-cells/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

1. A fresh-context `research` subagent reviews `docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/DECISION.md` (`D68 DECISION REVIEW: APPROVE`), and the decision log is posted to issue `#2883`.
2. `crates/uor-r4-discrete/src/objective.rs` implements `RankOccupancyPolicy` (`NearestAddressMarginV1` `word = 0`, `OccupiedCellRankHingeV1` `word = 1`, `FixedGridRankHingeV1` `word = 2`, `OccupiedCellRankCrowdHingeV1` `word = 3`), exact integer row-loss evaluation (`8*T + 32*R_grid + 8*C_crowd` with `u64` overflow status `12`), move-time `G-Occ` refusal (`occ >= 128`, `n(c) <= 32`), and finite controls `C-R1..C-R6` and `C-R7a..C-R7d` without modifying `discrete.cu` or `driver_source_sha256`.
3. `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs` implement `grade-rank-objective`, producing the pinned receipt `docs/labs/r-rank-objective-keeps-cells/rank-objective-grade.json` (`uor-r4.rank-objective-keeps-cells-grade/v1`) that evaluates `K16-A`, `K16-R`, `#2831` `Q1`, `#2836` `Stage T` (`T16_fz` and `T16_rel`), `#3267` within-cell decomposition, and `R_grid` / `R_occ` against `G-Occ`, `G-Adm`, and the count tables (`U = 1,226`, `T2 = 6,300`, `T(c(x)) = 4,085`, `U_restr(A_W) = 1,127`, `T_sym_restr(A_W) = 1,909`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`).
4. `docs/harness/prep/r-rank-objective-keeps-cells-2026-10-25.md`, `docs/labs/r-rank-objective-keeps-cells/README.md`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/r-rank-objective-keeps-cells.md` (`status: done`), `docs/roadmap/README.md`, and `docs/loop/tasks/r-rank-objective-keeps-cells/antigravity-97/REPORT.md` (`Closes #2883`) are updated and pass `uor-harness loop gates --full` and D66 fresh-context review.

## Stop and ask if

- Any finite control (`C-R1..C-R6`, `C-R7a..C-R7d`) or SHA-256 pin check fails: classify `instrument` vs `mechanism` first and decide via D68 without waiting.
- `discrete.cu` (`ORDINARY_WRITER`) or `driver_source_sha256` (`04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`) would be altered: keep all new rank-occupancy evaluation in `objective.rs` and `stage_t.rs`.
