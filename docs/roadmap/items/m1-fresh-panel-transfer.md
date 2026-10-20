---
id: m1-fresh-panel-transfer
title: Evaluate the gain on the fresh sealed panel
milestone: M1
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-export-gap-and-toks]
---

## Why it matters

Prevents a panel-fitted result from closing M1 (the M2 reply model failed that way).

## What is missing

M1 bar (e) needs the gain to hold on a fresh, sealed test panel registered in advance. The fresh slice is still sealed (fresh transfer NOT_TRIGGERED or NOT_RUN) and opens only after the earlier bars pass.

## Done when

'(e) Fresh transfer. The gain holds on a fresh, sealed test panel registered in advance.'

## How to start

Open these first:

- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/research/open-questions.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:14`](../../../ROADMAP.md)
- [`docs/integration/NOW.md:85`](../../../docs/integration/NOW.md)
- [`docs/start/where-were-going.md:34`](../../../docs/start/where-were-going.md)

