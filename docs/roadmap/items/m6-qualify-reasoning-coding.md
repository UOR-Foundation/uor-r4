---
id: m6-qualify-reasoning-coding
title: Qualify reasoning and coding on the served model
milestone: M6
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism]
---

## Why it matters

M6 closes reasoning and coding on the same model.

## What is missing

m6-reasoning is 0/1 qualified and not started. The served M4 model has to pass the frozen panel and the causal ablation.

## Done when

M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory.

## How to start

Open these first:

- `issue#2034 body`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:19`](../../../ROADMAP.md)
- issue#2034

