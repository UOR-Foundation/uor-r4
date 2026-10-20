---
id: m2-reply-grading-instrument-r14
title: Build the instrument that R14 says reply grading lacks
milestone: M2
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

A conversation bar cannot be scored without it.

## What is missing

The #2029 body says the 116/232 reply bars are BLOCKED under R14 with 'no instrument' (v5-reply stood at 37/232). Stage 5 and M2 need replies graded by fixed deterministic checks. The sources do not spell out the missing instrument, so scope it first.

## Done when

A fixed deterministic grading instrument for replies, with a recorded scope note on what R14 required and how this instrument meets it.

## How to start

Open these first:

- `docs/integration/DECISIONS.md (D39 item 1)`
- `docs/start/where-were-going.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/start/where-were-going.md:52`](../../../docs/start/where-were-going.md)
- issue#2029

