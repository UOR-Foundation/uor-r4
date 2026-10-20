---
id: m3-memory-turn-discriminator-gate
title: Add a memory-turn discriminator or margin gate
milestone: M3
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [m3-nonrelation-prose-panel]
---

## Why it matters

Prevents silent corruption of stored facts.

## What is missing

The required fix from #1718 is a discriminator or margin gate so that non-memory prose cannot compile to ASSERT or CORRECT, measured before qualifying M3. world=v2 uses compiler-save-op-v25-rawtable and op-model-v25.

## Done when

M3 criterion 3: 'Non-memory conversational prose does not silently overwrite stored facts (#1718)', measured on the pre-registered panel.

## How to start

Open these first:

- `relation_compiler.rs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1718)

