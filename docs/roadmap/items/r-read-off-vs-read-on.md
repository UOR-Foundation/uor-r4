---
id: r-read-off-vs-read-on
title: Compare read-off and read-on for Route and Flock on checkpoints
milestone: research
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

Decides whether the reads are worth carrying into the trunk.

## What is missing

Whether the Route and Flock reads add anything is untested; Route got only 20 proposals in 16,404 batches. This is a no-training comparison on existing checkpoints.

## Done when

Paired read-on versus read-off results on the same retained checkpoints recorded as notebook rows, with the classification instrument versus mechanism stated.

## How to start

Open these first:

- `crates/uor-r4-discrete/src/discrete.cu`
- `docs/labs/attention-state-2026-10-18/README.md`
- `docs/mechanisms/flock-read.md`
- `docs/mechanisms/route-read.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:38`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:89`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:43`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:44`](../../../docs/research/open-questions.md)

