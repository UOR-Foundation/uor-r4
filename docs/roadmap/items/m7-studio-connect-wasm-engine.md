---
id: m7-studio-connect-wasm-engine
title: Connect the Studio to the browser WASM engine
milestone: M7
area: serving
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Delivers the web client product as soon as a model exists.

## What is missing

The browser engine (serving-wasm: loadStack, step, saveSession, restoreSession) and the Studio page both exist but are not connected, and no real model is available to load. Website deploys are paused. It must run entirely client-side in a Web Worker with truthful capability reporting and zero remote model calls.

## Done when

M7 criterion 3: the Studio runs entirely client-side in a browser Web Worker with truthful capability reporting and zero remote model calls; the .wasm module passes scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap (7/7 PASS, mul=0, div=0, float=0).

## How to start

Open these first:

- `web/studio`
- `crates/uor-r4-integer/src/wasm_serve.rs`
- `docs/wasm-serving/wasm_serve_demo.html`
- `docs/wasm-serving/verify.mjs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`STATUS.md:22`](../../../STATUS.md)
- [`docs/start/where-were-going.md:74`](../../../docs/start/where-were-going.md)
- issue#2035

