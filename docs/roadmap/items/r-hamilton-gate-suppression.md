---
id: r-hamilton-gate-suppression
title: Settle whether a norm-preserving Hamilton gate can suppress a signal
milestone: research
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

The Hamilton gate is the planned feed-forward layer in the Stage 3 trunk.

## What is missing

The claim that a norm-preserving Hamilton gate can suppress a signal is a hypothesis, a reading of the dossier only. Nothing in the sources shows it was checked.

## Done when

A proof or finite test that settles whether such a gate can suppress a signal, labelled proof or measurement.

## How to start

Open these first:

- `docs/mechanisms/hamilton-gate.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/research/open-questions.md:97`](../../../docs/research/open-questions.md)

