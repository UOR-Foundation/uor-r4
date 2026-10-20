---
id: m1-export-gap-and-toks
title: Measure export gap and tokens per second on a trained model
milestone: M1
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-d11-exporter-discrete, a saved trained discrete model (A8 rerun, BVG-K or trunk)]
---

## Why it matters

M1 bar (d) is defined through the export, and M5 builds on the same measurement.

## What is missing

Tokens per second for a model trained under today's rules has never been measured, and the export gap is unmeasured. Stage 4 needs both prompt-reading and reply-writing rates on a laptop, and integer top-1 scored through the D11 export on the DEV panel.

## Done when

Stage 4 bar: 'the integer audits stay green, the gap stays within the noise band, and tokens per second is reported for both reading the prompt and writing the reply.'

## How to start

Open these first:

- `crates/uor-r4-integer`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/labs/wasm-serving-runtime-2026-10-18/README.md`
- `docs/mechanisms/lut-serving.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:94-97`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:136`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:165`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:23`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:94`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:55`](../../../docs/model/architecture.md)
- [`docs/start/where-we-are.md:58`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:50`](../../../docs/start/where-were-going.md)

