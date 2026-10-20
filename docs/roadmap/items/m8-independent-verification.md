---
id: m8-independent-verification
title: Independently verify the alpha on consumer hardware
milestone: M8
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m8-release-packaging]
---

## Why it matters

Confirms the alpha claims without the authors' own setup.

## What is missing

The M8 criteria require independent verification on consumer hardware of the one UORLUT01 artifact. Nothing exists to verify yet.

## Done when

A third party reproduces the install and the M1 to M7 bars on consumer hardware from the checksum-pinned artifact.

## How to start

Open these first:

- `issue#2036 body`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2036

