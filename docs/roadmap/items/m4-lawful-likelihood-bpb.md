---
id: m4-lawful-likelihood-bpb
title: Build a lawful likelihood so bits per byte can be scored
milestone: M4
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

M4's held-out quality bar and any later language-modelling comparison depend on it.

## What is missing

The current model's loss gives no probabilities, so the 0.90 BPB bar that M4 still carries cannot be scored on it. BPB returns once a lawful likelihood exists, and that is still unbuilt; the sources name no design.

## Done when

A likelihood with no softmax or squashing (D23, D25) feeding a registered instrument that scores the chat held-out stream in bits per byte, so the M4 bar 'held-out quality at or below 0.90 bits per byte' can be tested.

## How to start

Open these first:

- `CHARTER.md (4.8)`
- `docs/research/open-questions.md`
- `docs/mechanisms/lut-serving.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:23-24`](../../../ROADMAP.md)
- [`docs/research/open-questions.md:102`](../../../docs/research/open-questions.md)
- issue#2032

