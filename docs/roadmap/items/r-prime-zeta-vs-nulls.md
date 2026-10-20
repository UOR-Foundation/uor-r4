---
id: r-prime-zeta-vs-nulls
title: Test the prime-zeta code against injective nulls at zero updates
milestone: research
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

Tells whether the codebook can be replaced by any injective code.

## What is missing

It is unknown whether PrimeZeta's structure matters or only its uniqueness (it gives all 4,096 tokens distinct addresses). The comparison against random and quota-only injective nulls has not been run at zero updates.

## Done when

Zero-update reach and capacity numbers for prime-zeta, random-injective and quota-only-injective codes on the same panel, recorded as a notebook row.

## How to start

Open these first:

- `archive/labs/prime-zeta-construction-2026-10-14/README.md`
- `docs/mechanisms/prime-zeta-codebook.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:39`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:87`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:45`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:31`](../../../docs/start/where-we-are.md)

