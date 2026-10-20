---
id: r-lawful-pointer-forced-copy-oracle
title: Build a lawful pointer, then run the forced-copy oracle
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

Decides whether pointer-style copying can be learned lawfully.

## What is missing

The forced-copy oracle is the cheapest decisive pointer test, but it needs a lawful pointer first because running it on the old pointer would execute a banned model. No lawful pointer exists.

## Done when

A lawful pointer with finite controls, then the forced-copy oracle run once under a registered sheet.

## How to start

Open these first:

- `archive/labs/ptr-addr-2026-10-10/README.md`
- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/pointer-key.json`
- `docs/mechanisms/pointer-key.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:54`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:114`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:60`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:32`](../../../docs/start/transformer-vs-geometry.md)

