---
id: m1-wider-code-carrier
title: Prototype a wider code carrier (more roots or blocks)
milestone: M1
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-address-key-full-vector]
---

## Why it matters

Settles whether vector width, not the address key, caps how many words the addresses can separate.

## What is missing

Whether the 16-bit carrier limits capacity is untested. A wider code is optional in Stage 1 and needs new GPU kernel code and a fresh CUDA qualification. The sources name no design or sheet.

## Done when

A wider-carrier build passes finite CUDA qualification and a zero-training capacity measurement against the Stage 1 bar (clear 1,941; start above 1,351 of 3,847).

## How to start

Open these first:

- `crates/uor-r4-discrete (discrete.cu)`
- `crates/uor-r4-discrete/src/discrete.cu`
- `docs/mechanisms/discrete-learning.md`
- `docs/model/architecture.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:121`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:37`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:23`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:43`](../../../docs/research/open-questions.md)
- [`docs/start/where-were-going.md:44`](../../../docs/start/where-were-going.md)

