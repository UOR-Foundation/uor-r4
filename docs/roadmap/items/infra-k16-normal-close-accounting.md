---
id: infra-k16-normal-close-accounting
title: Record K16 normal close and final resource accounting
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

Closes custody and cost records for the last full experiment so later runs build on a clean ledger.

## What is missing

NOW's historical K16 snapshot says the same lease is retained and normal close and final resource accounting are not yet recorded. The registered outcome call itself is done (OCCUPANCY_COLLAPSE_TRIGGERED, 2026-10-19), so check whether closure is already recorded before starting; this line may be stale.

## Done when

Normal close and final resource accounting recorded for the K16 run, or the stale statement in NOW removed with a pointer to where it was recorded.

## How to start

Open these first:

- `docs/labs/k16-four-role-execution-2026-10-18/README.md`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:78-81`](../../../docs/integration/NOW.md)

