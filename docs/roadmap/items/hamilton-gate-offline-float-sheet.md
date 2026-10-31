---
id: hamilton-gate-offline-float-sheet
title: Run a Hamilton gate offline-float sheet with language fit and export gap
milestone: research
area: mechanism
status: done
open_to: lab
needs_gpu: true
depends_on: [float-training-module, r-secant-surrogate-descent]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/**, crates/uor-r4-integer/**, docs/mechanisms/hamilton-gate.md, docs/harness/prep/hamilton-gate-offline-float-sheet*, docs/labs/hamilton-gate-offline-float-sheet/**, docs/loop/tasks/hamilton-gate-offline-float-sheet/**]
---

## Why it matters

The offline-float sheet completed language-fit and export-gap measurements. The Hamilton gate is ACTIVE and trained in the discrete M1 composition and on the FBS trunk, whose unsnapped exact-unit-quaternion form exports and serves under `lut-stack/5`, with exact rotations and the export gap inside its bar at DEV scope. Per-block suppression is measured: 0 of 78,888,960 blocks near the guard core and 0 of 986,112 downstream rows below 0.8, with a mixed language-fit result and `G-Hamilton-ExportGap = PASS`.

## What is missing

Remaining work is identity-preconditioned/residual initialization at 1,024 steps or more, a matched full-rung comparison, and cross-block/downstream behavior ([mechanism record](../../mechanisms/hamilton-gate.md), [lab record](../../labs/hamilton-gate-offline-float-sheet/README.md)).

## Done when

An offline-float (D37) sheet reports a language fit on the integer DEV metrics and an export gap, and states whether the gate suppressed signal.

## How to start

Open these first:

- `docs/mechanisms/hamilton-gate.md`
- `docs/labs/hamilton-gate-offline-float-sheet/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:135`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:168`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:17`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:101`](../../../docs/mechanisms/README.md)

