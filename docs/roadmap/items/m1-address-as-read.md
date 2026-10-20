---
id: m1-address-as-read
title: Build the address-as-read mechanism and an object-store consumer
milestone: M1
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Connects the exact storage system to the model's read path, which M2 grounding needs.

## What is missing

Address as the read has never been built, and nothing yet consumes the object store. The object store is delivered (60 tests, a conformance check pulling 262,144 bytes back byte-identical).

## Done when

A read that resolves an address to stored content from the object store, with finite tests, and a registered test of whether it improves ranking.

## How to start

Open these first:

- `archive/labs/addressed-memory-2026-10-10/README.md`
- `archive/labs/d19-object-store-2026-10-15/README.md`
- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/exact-memory.json`
- `docs/labs/uorobj01-conformance-2026-10-18/README.md`
- `docs/mechanisms/README.md`
- `docs/mechanisms/exact-memory.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:113`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:68`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:86`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:33`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:34`](../../../docs/start/where-we-are.md)

