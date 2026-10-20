---
id: m4-export-uorlut01-cli
title: Serve the chosen model as UORLUT01 through the uor-chat CLI
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-d11-exporter-discrete, m4-lawful-likelihood-bpb, m1-close-saved-model, m3-qualify-memory]
---

## Why it matters

Unifies the M1 to M3 model into the one served artifact that M5 to M8 depend on.

## What is missing

One chosen model must be exported as a single UORLUT01 file, serve through the uor-chat CLI, report its training-to-D11 export gap and meet the held-out bar; none exists. The native and WASM serving audits pass on today's build and must stay at 0 float, 0 softmax or squashing, 0 matmul and 0 mul/div on the final build. M4 is held.

## Done when

M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'; export gap reported (D37); static audits green; fresh-process restore bit-identical.

## How to start

Open these first:

- `crates/uor-r4-integer`
- `scripts/audit_zero_matmul_serving.py`
- `scripts/audit_zero_matmul_wasm.py`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:17`](../../../ROADMAP.md)
- issue#2032

