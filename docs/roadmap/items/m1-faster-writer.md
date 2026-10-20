---
id: m1-faster-writer
title: Replace the full-vocabulary writer scan
milestone: M1
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

Per-token parameter sparsity (D5) and any move past 4,096 tokens depend on it.

## What is missing

The writer scans the whole vocabulary, about 5 million comparisons per row, which breaks the per-token sparsity goal at larger vocabularies (proof, by count, architect review 3.5). No faster writer design exists. It needs new kernel work and a CUDA qualification.

## Done when

A writer whose measured per-row comparison count is below the full-vocabulary scan, with finite controls matching the full-scan writer and a CUDA qualification.

## How to start

Open these first:

- `crates/uor-r4-discrete`
- `crates/uor-r4-discrete/src/discrete.cu`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/mechanisms/admission.md`
- `docs/mechanisms/tokenizer.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:170`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:19`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:68`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:88`](../../../docs/research/open-questions.md)

