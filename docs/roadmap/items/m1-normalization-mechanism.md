---
id: m1-normalization-mechanism
title: Design a lawful normalization mechanism
milestone: M1
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

A required slot in the Stage 3 trunk.

## What is missing

No normalization is designed, and the current model has none. It must meet the charter bans on softmax-like squashing and matmul.

## Done when

A normalization mechanism with finite controls obeying CHARTER 3, plus a pre-registered test of its effect in the trunk.

## How to start

Open these first:

- `docs/mechanisms/README.md`
- `docs/mechanisms/hamilton-gate.md`
- `docs/start/transformer-vs-geometry.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:18`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:142`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:64`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:85`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:53`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:60`](../../../docs/start/where-we-are.md)

