---
id: legal-construction-exact-solve
title: Solve the saved legal-construction case in exact arithmetic
milestone: research
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

It reopens a PARKED optimizer path that needs no gradients.

## What is missing

Legal construction picks each parameter change from values the integer model can hold, but it has never been scored: every attempt stopped on solver numerics. The saved case has 1,960 variables and has not been solved in exact arithmetic. It is a host solver task, so no GPU is assumed.

## Done when

The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as scored or infeasible.

## How to start

Open these first:

- `docs/mechanisms/legal-construction.md`
- `archive/labs/direct-legal-construction-2026-10-09/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/mechanisms/README.md:111`](../../../docs/mechanisms/README.md)

