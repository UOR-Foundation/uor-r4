---
id: m1-bvg-k-stage-t-training
title: Train the BVG-K batched-coordinate law on two seeds
milestone: M1
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-w2-capacity-capture, m1-bvg-k-block-apply-c2, m1-a8-full-dose-rerun, m1-w2-code-utilization, m1-address-key-full-vector]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-bvg-k-stage-t-training*, docs/labs/m1-bvg-k-stage-t-training/**, docs/loop/tasks/m1-bvg-k-stage-t-training/**]
---

## Why it matters

Stage 2, 'pick the learning law': the first learning-law candidate that stays whole-number.

## What is missing

**Run outcome: complete; the Stage 2 writer bar was not met.** Stage T ran on 25 October with D67/D68 PASS and all five Step 0 controls passing. On `T16_fz`, BVG-K raised full-DEV admission to 5,009 of 14,287 with G-Adm and 256/256 G-Occ PASS; checkpoint-restricted tables reached `T_restr(A_W) = 2,140` and `G_restr(A_W) = 3,084`. The writer scored 455 on `T16_fz` and 371 on `T16_rel`, below unigram 1,226, so `stage2_bar_met_two_seeds = false`. The registered result is `DIAGNOSTIC_STOP_CONSECUTIVE_REJECTIONS`, `mechanism_negative = false`, not a Stage 2 pass or an M1 close by itself ([Stage T record](../../labs/m1-bvg-k-stage-t-training/README.md)).

**Original pre-run status, retained only as history:** Stage T waited on #2834, seed-2 configs, block evidence, Step 0 controls and checkpoint-restricted W2 tables. Those instruments and the two-seed runs are now complete; their exact limitations remain in the sealed record.

## Done when

**Original acceptance text, unchanged:** Stage 2 bar: 'beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds.'

**Observed outcome:** the two-seed run is complete at its registered diagnostic-stop scope, but the writer bar above was not met. The later saved-model M1 close is a separate evidence record, not a retroactive Stage T pass.

## How to start

**Historical pre-run procedure; Stage T has run.**

Open these first:

- `docs/integration/NOW.md`
- `docs/labs/bvgk-rowsk-scoring-2026-10-18`
- `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`
- `docs/mechanisms/discrete-learning.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [Stage T record](../../labs/m1-bvg-k-stage-t-training/README.md) and [antigravity-84 REPORT](../../loop/tasks/m1-bvg-k-stage-t-training/antigravity-84/REPORT.md), including the two-seed diagnostic stop and unmet writer bar
- [W2 code-utilization record](../../labs/m1-w2-code-utilization/README.md)
- [Current open-questions record](../../research/open-questions.md) and [where we are going](../../start/where-were-going.md)
- [19 October pre-run audit](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md), retained as history

