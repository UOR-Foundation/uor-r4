---
id: m6-exact-arithmetic-mechanism
title: Design exact arithmetic on the geometric path
milestone: M6
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Reasoning and coding both need exact computation.

## What is missing

Exact arithmetic is named as the visible gap for reasoning and coding, but the sources specify no mechanism. It must stay on the same geometric stack with typed operators (SpiralCore, icosian transport) and obey the bans on matmul and squashing.

## Done when

A mechanism with finite controls on exact arithmetic items, integrated through the shared operators with no second model or external teacher.

## How to start

Open these first:

- `docs/mechanisms/spiralcore-fbs.md`
- `docs/start/transformer-vs-geometry.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`STATUS.md:21`](../../../STATUS.md)
- [`docs/start/where-were-going.md:63`](../../../docs/start/where-were-going.md)
- issue#2034

