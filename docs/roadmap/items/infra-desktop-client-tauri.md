---
id: infra-desktop-client-tauri
title: Build the Tauri desktop client against the synthetic stand-in
milestone: infra
area: serving
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m7-studio-connect-wasm-engine]
---

## Why it matters

The third product the owner defined on 19 October.

## What is missing

The desktop client exists only as a written specification: a Tauri app sharing the web interface, running the model natively with durable memory and offline use on M1-class laptops. It is a defined product but is not part of any M1 to M8 bar in these sources.

## Done when

A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification.

## How to start

Open these first:

- `docs/products/desktop-client.md`
- `docs/products/README.md`
- `web/studio`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/start/where-were-going.md:67-77`](../../../docs/start/where-were-going.md)

