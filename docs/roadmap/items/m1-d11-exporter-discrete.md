---
id: m1-d11-exporter-discrete
title: Write the exporter from a trained discrete model to a D11 file
milestone: M1
area: serving
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Needed for M1 bar (d), Stage 4 and M4; today the serving engine runs only a synthetic stand-in.

## What is missing

No exporter turns a trained discrete model into a D11 (UORLUT01) file; older float stacks were exported, nothing trained under today's rules has been. A lawful exporter for any trained trunk is also missing, so the export gap is unmeasured.

## Done when

Exporter on main; a discrete snapshot round-trips through crates/uor-r4-integer with matching integer outputs on finite tests; the serving audits stay green (0 float, 0 mul/div).

## How to start

Open these first:

- `archive/labs/d11-serving-2026-10-10/README.md`
- `crates/uor-r4-discrete`
- `crates/uor-r4-integer`
- `docs/integration/project-track.md`
- `docs/labs/wasm-serving-runtime-2026-10-18/README.md`
- `docs/mechanisms/lut-serving.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:23`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:94`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:53`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:87`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:61`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:58`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:50`](../../../docs/start/where-were-going.md)

