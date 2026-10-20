---
id: m1-a8-registration-and-runbook-staging
title: Register A8 finite qualification and stage the dose runbook
milestone: M1
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

It is the precondition for asking the owner for the A8 finite-qualification release.

## What is missing

NOW names audited-dose runbook and executor staging, plus a separate prospective finite-qualification registration, as current work. Registration must preserve the immutable scientific FIT sheet identity; PR #2785 tracks sheet restoration and the separate qualification binding. The A8 qualification and dose keep the sheet-bound source fd409fcdf, not later main.

## Done when

#2785 merged; finite-qualification sheet registered with its own binding and the original FIT sheet identity unchanged; runbook and executor staged; the exact owner-release command staged for Casey to type.

## How to start

Open these first:

- `docs/labs/admission-anchor-controller-2026-10-19/README.md`
- `docs/labs/address-cells-consumer-verifier-2026-10-19/README.md`
- `docs/labs/a8-training-configs-2026-10-19/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:18-34`](../../../docs/integration/NOW.md)

