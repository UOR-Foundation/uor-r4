---
id: m1-position-mechanism
title: Design a lawful position mechanism
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

No position mechanism is assigned; the only related result is about recurrent state and is confounded. The current model has no position mechanism, so the Stage 3 trunk cannot be assembled.

## Done when

A position mechanism with finite controls that obeys CHARTER 3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order.

## How to start

Open these first:

- `ROADMAP.md`
- `archive/integration/transport-attribution-d1-2026-09-28.md`
- `docs/mechanisms/recurrence-gates.md`
- `docs/research/findings.md`
- `docs/start/transformer-vs-geometry.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:15`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:132`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:63`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:84`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:51`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:60`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:48`](../../../docs/start/where-were-going.md)

