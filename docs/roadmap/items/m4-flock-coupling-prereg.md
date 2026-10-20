---
id: m4-flock-coupling-prereg
title: Finish the flock-coupling pre-registration
milestone: M4
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

The read line that M4 originally selected.

## What is missing

The M4 issue (18 October) names a draft C1 flock-coupling pre-registration, S1 flock transport mean plus S2 FBS tree (the Murmuration Read), gated on Rung 0 and owner hold release. The 19 October renormalization may have changed which read is active, so confirm it is still wanted.

## Done when

A registered C1 flock-coupling sheet with a non-author FIT, or a recorded decision that it is superseded.

## How to start

Open these first:

- `archive/labs/discrete-driver-cost-2026-10-13/README.md`
- `crates/uor-r4-discrete/src/discrete.cu`
- `docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md`
- `docs/mechanisms/flock-read.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:38`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:90`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:77`](../../../docs/model/architecture.md)
- issue#2032

