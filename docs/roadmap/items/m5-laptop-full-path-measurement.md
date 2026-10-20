---
id: m5-laptop-full-path-measurement
title: Measure M1-class laptop cost on the accepted served model
milestone: M5
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: [m4-export-uorlut01-cli]
---

## Why it matters

An integer path is not an efficiency result until it is measured (CHARTER 3.9).

## What is missing

m5-cost is 0/1 qualified. Only synthetic-stack measurements exist: macmon M1 runs (4.13 vs 0.96 mJ/tok) and a provisional Linux RAPL receipt (17,190 tok/s, 3.172 mJ/tok gross, 1.530 mJ/tok net). The owner's M1 run on the accepted M4 artifact is the authoritative record.

## Done when

M5 bar: 'Measured energy per token, memory and tokens per second on an M1-class laptop'; latency (ms/tok), peak RSS, energy (mJ/tok) and weights_per_token over total parameters measured and reproducible across ingestion, routing, memory lookup and emission, with no hidden host float fallback or uncounted external daemon.

## How to start

Open these first:

- `ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`
- `ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md`
- `ops/measurement_report_rnd.json`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:18`](../../../ROADMAP.md)
- issue#2033

