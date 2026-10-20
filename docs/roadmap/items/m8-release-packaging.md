---
id: m8-release-packaging
title: Prepare the checksummed alpha release package and install path
milestone: M8
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser]
---

## Why it matters

The deliverable of M8.

## What is missing

No versioned, checksum-pinned release artifact, reproducible install path or release notes exist. Release notes must bind the exact measured scores and known limitations, and the owner declares alpha.

## Done when

M8 bar: 'One versioned, checksum-pinned model file, declared alpha by the owner', with a reproducible installation and release notes binding the exact measured scores and known limitations.

## How to start

Open these first:

- `ROADMAP.md`
- `issue#2036 body`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:21`](../../../ROADMAP.md)
- issue#2036

