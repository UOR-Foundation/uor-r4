---
id: m1-a8-full-dose-rerun
title: Run the full A8 dose with reach protected
milestone: M1
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-a8-finite-cuda-qualification]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-a8-full-dose-rerun*, docs/labs/m1-a8-full-dose-rerun/**, docs/loop/tasks/m1-a8-full-dose-rerun/**]
---

## Why it matters

Tests whether reach survives training, the then-current D39 G-Adm condition, and gives the first run in which ranking can be judged with admission intact. G-Adm is upstream/historical evidence, not the approved close-grade M1 bar (b), which is exact export/reference agreement and B4.

## What is missing

The admission-preserving A8 rerun ran on 25 October 2026 with D67/D68 PASS. FrozenZero reached Q1 with `1,106 / 3,847`, `4,292 / 14,287` and `871 / 14,287`, then continued to a diagnostic stop at batch 5,248 under the 64-batch whole-interval rejection cadence; Relation stopped at batch 1,600. This A8 run did not open Fresh V2; the later M1 bar-(e) run opened and scored it at 7,215 / 7,228. This is an instrument/cadence-granularity stop, not a mechanism negative or an A8 `5,009` result; that number belongs to Stage T.

## Done when

Paired DEV verdict recorded for the registered arms and seeds; reach kept under the then-current D39 G-Adm condition or a registered diagnostic stop classified as instrument versus mechanism; this A8 run leaves the fresh slice sealed; the later M1 bar-(e) opening and score are recorded separately.

## How to start

Open these first:

- `docs/labs/a8-dose-2026-10-19/README.md`
- `docs/labs/a8-training-configs-2026-10-19/README.md`
- `docs/labs/address-cells-preflight-2026-10-19/README.md`
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

- [Historical `NOW.md` snapshot, lines 26-30](../../../archive/integration/NOW-2026-10-22.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:14`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:32`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:88`](../../../docs/mechanisms/README.md)
- [A8 full-dose rerun record](../../labs/m1-a8-full-dose-rerun/README.md)
- [`docs/labs/m1-a8-full-dose-rerun/README.md`](../../../docs/labs/m1-a8-full-dose-rerun/README.md)

