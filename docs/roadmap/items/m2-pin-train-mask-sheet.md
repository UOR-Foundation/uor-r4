---
id: m2-pin-train-mask-sheet
title: Pin the M2 TRAIN-mask, SHA, CLI and source sheet
milestone: M2
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

A sheet ready for FIT shortens the path once the owner decides on the M2 hold.

## What is missing

The 18 October next step is mechanical pinning of the masked TRAIN reference (source 2fd6358bb passes 11 finite controls) into a sheet before any GPU training or fresh-transfer evaluation. M2 training and rental stay separately HELD.

## Done when

A sheet pinning the TRAIN mask, SHA, CLI and source, with a non-author FIT, ready for an owner hold-release decision.

## How to start

Open these first:

- `docs/labs/m2-masked-train-reference-2026-10-13/README.md`
- `crates/uor-r4-core/src/native_geometric/learner`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2030

