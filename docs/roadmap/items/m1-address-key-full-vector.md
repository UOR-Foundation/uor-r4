---
id: m1-address-key-full-vector
title: Replace the 5-value address key with a full-vector metric
milestone: M1
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-w2-capacity-capture]
---

## Why it matters

Tests the leading hypothesis for the low top-1 and is Stage 1, 'make the capacity measurable'.

## What is missing

The address key uses only the first and last block and takes only 5 values (0, 4, 8, 12, 16) to order up to 256 cells (proof, from discrete.cu), so which cells open is largely decided by arbitrary numbering. The architect's likely lever is a full-vector or interior-block address metric, built and measured at zero training. It needs new kernel code and a fresh CUDA qualification.

## Done when

Stage 1 bar: 'enough distinct vectors that the number of positions still winnable clears the run bar of 1,941, and a starting reach above the address-cell model's 1,351 of 3,847', measured before any training.

## How to start

Open these first:

- `address_cells.rs`
- `attention_preflight.rs`
- `coverage_census.rs`
- `crates/uor-r4-discrete (discrete.cu)`
- `crates/uor-r4-discrete/src/discrete.cu`
- `docs/labs/address-cells-preflight-2026-10-19/README.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/mechanisms/discrete-learning.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:120-123`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:155`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:36`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:86`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:42`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:53`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:44`](../../../docs/start/where-were-going.md)

