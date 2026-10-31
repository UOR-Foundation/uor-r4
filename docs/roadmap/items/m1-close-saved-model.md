---
id: m1-close-saved-model
title: Close M1 bars (a)–(e) with one saved discrete model
milestone: M1
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training, m1-export-gap-and-toks]
scope: [crates/uor-r4-discrete/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md]
notebook: docs/research/notebook/m1-discrete-attention.md
---

## Why it matters

This item closes M1's bars (a)–(e); the separate `m1-heldout-phrasing-transfer` item closes bar (f). Together they unblock M2, M4 and everything after.

## What is missing

**Outcome: complete.** K16-A's dated 65 and 463 of 14,287 sat below unigram 1,226 and T2 6,300; K16-A failed G-Adm on both seeds and K16-R collapsed to 19 or 20 occupied cells. The 116/232 reply bar was BLOCKED under R14 and was not a closing condition.

**Completed 30 Oct 2026:** antigravity-110's two seeded `UORLUT01` artifacts pass bars (a)–(e): self-contained D11 loading; exact 14,287-row export agreement and B4; laptop tokens/s; DEV 7,002 / 7,001 against unigram 1,226 and T2 6,300, above the required 6,324 target (T2 + 24); and Fresh V2 7,215 / 7,228 against unigram 1,279 and T2 6,582 with the paired band. The older close-grade bar (c) reports 11,026 / 10,573 tok/s; the separate bar-f remeasurement reports 10,876 / 10,566 tok/s on Apple M1. The separate `m1-heldout-phrasing-transfer` item then passed bar (f): p-f 7,850 / 7,872 against T2 7,375, band 81. M1 #2029 is closed at all six bars. The 0.625 p-f reference-reply overlap with TRAIN means this is not a useful-chat claim.

**Upstream evidence retained (26 Oct 2026, `antigravity-100` #3293).** All upstream M1 building blocks and evaluators were on `main`: the A8 full-dose rerun (`#2831`) reached Q1 `4,292 / 14,287`, `1,106 / 3,847` and `871 / 14,287` before a diagnostic stop; Stage T (`#2836`, `#3239`) reached `5,009 / 14,287` DEV admission (`G-Adm` PASS, `256 / 256` occupied cells `G-Occ` PASS) with checkpoint-restricted `T_restr(A_W) = 2,140` and `G_restr(A_W) = 3,084`; `m1-within-cell-writer` (`#3267`, `#3280`) proved that `90.48%`–`92.18%` of missed `T_restr` hits lose within their own cell under `OuterL1V1`; `r-rank-objective-keeps-cells` (`#2883`, `#3290`) is `done`; `m1-export-gap-and-toks` (`#2839`, `#3287`) qualified exact zero `UORLUT01` export gap (`14,287 / 14,287` row agreement) and `232,126–232,879` response `tokens/s` on 1x RTX 5090; and `m1-fresh-panel-transfer` (`#2841`, `#3292`) landed the `d11-scorer fresh-score` gate and `TEST FITNESS: FIT` sheet. The completed close-grade contract above is this item's acceptance record; the older D39 conditions below are upstream/historical, not a second conflicting bar.

## Done when

One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

**Approved close-grade contract, 30 Oct 2026:** [decision log 6515151261](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6515151261) approves the completed outcome above: (a) self-contained D11 loading, (b) exact export agreement and B4, (c) tokens/s, (d) DEV integer top-1 above both count tables with target 6,324 = T2 + 24, (e) Fresh V2 transfer, and the separate `panel-score.json` for (f), held-out-phrasing transfer. The older D39 starting-reach, G-Adm and G-Occ conditions in this Done-when list remain upstream/historical requirements, not a relabelling of the close-grade fields. This records the approved close, not a new acceptance rule.

## How to start

Open these first:

- `CHARTER.md (4.8)`
- `docs/integration/DECISIONS.md (D39)`
- `docs/start/where-were-going.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:14`](../../../ROADMAP.md)
- [`docs/start/where-were-going.md:26-36`](../../../docs/start/where-were-going.md)
- issue#2029

