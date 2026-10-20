---
id: object-store-consumer-m3
title: Wire the object store into the model and run the durable-memory bars
milestone: M3
area: architecture
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

It delivers M3's durable memory and gives later replies exact storage to read from.

## What is missing

The object store (D19) keeps bytes exactly in a hashed tree and has 60 tests and a client conformance pass, but nothing consumes it. The durable-memory bars have not run. No consumer exists on the model path.

## Done when

A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results.

## How to start

Open these first:

- `docs/mechanisms/object-store.md`
- `archive/labs/d19-object-store-2026-10-15/README.md`
- `docs/labs/uorobj01-conformance-2026-10-18/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:104`](../../../docs/mechanisms/README.md)
- [`docs/start/transformer-vs-geometry.md:34`](../../../docs/start/transformer-vs-geometry.md)

