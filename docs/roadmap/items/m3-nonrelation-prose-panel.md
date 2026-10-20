---
id: m3-nonrelation-prose-panel
title: Build a pre-registered non-relation prose panel
milestone: M3
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

M3 acceptance requires that non-memory prose never overwrites stored facts.

## What is missing

In world=v2, ordinary small talk can compile as ASSERT or CORRECT (an example in #1718 compiled to CORRECT on relation 4 and overwrote a stored fact). No pre-registered set exists to measure false-assert and false-overwrite rates.

## Done when

A pre-registered non-relation prose panel with grading, plus the baseline false-assert and false-overwrite rates on world=v2.

## How to start

Open these first:

- `relation_compiler.rs`
- `stack_grounded_session.rs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1718)

