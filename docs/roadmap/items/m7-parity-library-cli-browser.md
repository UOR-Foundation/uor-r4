---
id: m7-parity-library-cli-browser
title: Build the library, CLI and browser bit-identity parity harness
milestone: M7
area: serving
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m7-studio-connect-wasm-engine, m4-export-uorlut01-cli]
---

## Why it matters

It is the M7 closing bar and also tests fresh-process restore identity.

## What is missing

No harness yet compares tokens and session snapshots across the Rust library (IntegerCapabilityApi), the uor-chat CLI and the browser worker on one artifact. The Node save and restore check passes only on a synthetic stand-in.

## Done when

M7 bar: 'giving bit-identical tokens across the library, command line and browser', with identical session snapshots, on the accepted UORLUT01 artifact.

## How to start

Open these first:

- `crates/uor-r4-integer/src/capability_api.rs`
- `crates/uor-r4-integer/tests/capability_api_tests.rs`
- `docs/wasm-serving/verify.mjs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:20`](../../../ROADMAP.md)
- issue#2032
- issue#2035

