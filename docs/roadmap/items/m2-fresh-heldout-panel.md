---
id: m2-fresh-heldout-panel
title: Build a genuinely fresh held-out reply panel for M2
milestone: M2
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Without a fresh panel the 40% bar cannot even be attempted.

## What is missing

The saved parent scored 437/512 on the development panel but 0/128 on the opened transfer diagnostic; the audit calls the reply model a panel-fitting failure on rows already seen. A genuinely fresh test is NOT_RUN. The M2 bar needs at least 40% on fresh questions with the existing graders unchanged (D22).

## Done when

A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

## How to start

Open these first:

- `docs/labs/m2-masked-train-reference-2026-10-13/README.md`
- `docs/research/open-questions.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`STATUS.md:17`](../../../STATUS.md)
- [`docs/research/open-questions.md:74`](../../../docs/research/open-questions.md)
- issue#2030

