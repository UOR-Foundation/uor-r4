---
id: m6-reasoning-panel-freeze
title: Build and freeze the m6-reasoning panel and oracles
milestone: M6
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

The M6 bar cannot be attempted without a frozen panel and oracles.

## What is missing

The frozen m6-reasoning panel does not exist: it must cover unseen multi-step compositions, altered constraints, and executable coding tasks checked by deterministic compilation or test oracles (cargo test, exact output). Status is not started.

## Done when

A frozen, pre-registered panel with deterministic oracles for compositions, altered constraints and code tasks (D28: no external teacher).

## How to start

Open these first:

- `issue#2034 body`
- `CHARTER.md (3.10)`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:19`](../../../ROADMAP.md)
- issue#2034

