---
id: float-training-module
title: Create the separate scanned D37 float training module
milestone: infra
area: architecture
status: ready
open_to: lab
needs_gpu: false
depends_on: [owner-decision-float-path-scope]
---

## Why it matters

It is the prerequisite for W4 and for any trunk built from lawful parts (Stage 3).

## What is missing

D37 float training is legal but unused. It must live in a separate module with its own scanned allowlist entry (D37 item 6), never crates/uor-r4-training, which carries the historical dense and sigmoid stack and is listed as held. No such module exists, and the 24-kernel extract is qualified at finite controls only.

## Done when

A new module builds on the laptop with CPU tests, passes the banned-token scan under its own allowlist entry, and compiles the 24-kernel extract without crates/uor-r4-training.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/mechanisms/geometric-stack.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:112`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:130`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:53`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)

