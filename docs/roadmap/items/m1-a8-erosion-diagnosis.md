---
id: m1-a8-erosion-diagnosis
title: Diagnose A8 reach erosion from retained checkpoints
milestone: M1
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

Tells the controller design whether the loss comes from the update moves, the read or the instrument.

## What is missing

Claude owns a host-only diagnosis of why Relation reach fell from 1,351 to 551 at Q1, using retained checkpoints and move logs. It recomputes no admission and so far establishes no causal admission-loss conclusion (see the latest attribution correction on #2029). No paired DEV verdict, KEEP or fresh success exists.

## Done when

A diagnosis note from the retained checkpoints and move logs that states which causal claims it does and does not support; recorded as one notebook row.

## How to start

Open these first:

- `docs/labs/a8-dose-2026-10-19/README.md`
- `docs/labs/address-cells-preflight-2026-10-19/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:38-43`](../../../docs/integration/NOW.md)
- [`docs/start/where-we-are.md:56`](../../../docs/start/where-we-are.md)

