---
id: m6-causal-ablation-instrument
title: Build the intermediate-state ablation instrument for M6
milestone: M6
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: [m6-reasoning-panel-freeze]
---

## Why it matters

It is half of the M6 acceptance criteria.

## What is missing

M6 needs causal intermediate-state ablation showing reasoning and code generation depend on geometric state transport and addressed memory rather than memorized templates. No instrument exists. It needs model hooks that depend on the trunk's state design.

## Done when

An ablation instrument registered before any run, able to knock out state transport and addressed memory and report the effect on the frozen panel.

## How to start

Open these first:

- `issue#2034 body`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2034

