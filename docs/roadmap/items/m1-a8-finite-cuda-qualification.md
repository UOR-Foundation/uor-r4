---
id: m1-a8-finite-cuda-qualification
title: Run A8 admission-anchor controller CUDA qualification and restoration
milestone: M1
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-a8-registration-and-runbook-staging]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-a8-finite-cuda-qualification*, docs/labs/m1-a8-finite-cuda-qualification/**, docs/loop/tasks/m1-a8-finite-cuda-qualification/**]
---

## Why it matters

Qualifies the version of A8 that protects reach before any full-dose rerun; the first A8 dose lost its reach at the Q1 stop.

## What is missing

**Outcome: complete at the finite-qualification scope.** The admission-preserving controller and qualification producer are on main (#2782, #2783). Real CUDA qualification passed all `5/5` stage checks on both Relation and FrozenZero, with restoration and timing recorded under D67 and the sheet-bound source `fd409fcdf`. The timing projection triggered the registered instrument-class gate-cost stop and required the full-dose replan; it was not a mechanism negative. The subsequent full-dose diagnostic run is recorded separately, and the next admission-preserving rerun is **NOT_RUN**.

## Done when

All finite qualification controls PASS on CUDA with restoration and timing recorded, under the sheet-bound source fd409fcdf. **Completed:** both arms passed `5/5`; see the qualification record below.

## How to start

Open these first:

- `docs/labs/a8-dose-2026-10-19/README.md`
- `docs/labs/admission-anchor-controller-2026-10-19`
- `docs/labs/admission-anchor-controller-2026-10-19/README.md`
- `docs/mechanisms/address-cells.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [Finite CUDA qualification record](../../labs/m1-a8-finite-cuda-qualification/README.md) and [antigravity-56 REPORT](../../loop/tasks/m1-a8-finite-cuda-qualification/antigravity-56/REPORT.md) (`5/5` on both arms, restoration and timing)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:32`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:88`](../../../docs/mechanisms/README.md)
- [`docs/start/where-we-are.md:56`](../../../docs/start/where-we-are.md)

