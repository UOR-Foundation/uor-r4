---
id: m1-bvg-k-block-apply-c2
title: Finish BVG-K Stage C2 (block apply) qualification
milestone: M1
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

BVG-K is the faster discrete learning law that Stage 2 tries first if W2 blames the learning law.

## What is missing

BVG-K Stage C1 is qualified (K-coordinate scoring exact 5,130/5,130, sabotage FAILs as required, K=16 scores 16 coordinates per launch at 16.25x throughput). NOW listed Stage C2 (block apply) as next on 2026-10-18, and a 10-19 lab record is titled 'injective-preflight-and-c2-qualify', so check whether C2 is already qualified. Stage T training stays held.

## Done when

Stage C2 finite controls PASS (block apply exact) and are recorded; Stage T sheet ready for FIT.

## How to start

Open these first:

- `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`
- `docs/mechanisms/discrete-learning.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:66`](../../../docs/integration/NOW.md)
- [`docs/start/where-were-going.md:46`](../../../docs/start/where-were-going.md)

