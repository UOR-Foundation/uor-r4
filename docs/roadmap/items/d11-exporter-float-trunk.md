---
id: d11-exporter-float-trunk
title: Write the D11 exporter for a D37 float trunk
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [float-training-module, d11-table-native-exporter, owner-decision-float-path-scope]
---

## Why it matters

Without it no float-trained trunk can be served, so Stages 3–5 stop at training.

## What is missing

The recurrence mode is refused by export and QAT, the Hamilton gate still needs the icosian snap, and training/src/stack_export.rs is compiled out with #[cfg(any())]. Integer schemas /1–/2 serve softmax. No exporter exists for a float trunk.

## Done when

A float-trunk snapshot with recurrence and the Hamilton gate exports to D11 with the audits green and the export gap measured inside the band.

## How to start

Open these first:

- `training/src/stack_export.rs`
- `docs/mechanisms/lut-serving.md`
- `docs/mechanisms/recurrence-gates.md`
- `docs/mechanisms/hamilton-gate.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:135`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:66`](../../../docs/model/architecture.md)
- [`docs/start/transformer-vs-geometry.md:61`](../../../docs/start/transformer-vs-geometry.md)

