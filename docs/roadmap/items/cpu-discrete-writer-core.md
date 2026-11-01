---
id: cpu-discrete-writer-core
title: Build an exact CPU discrete writer and admission oracle
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-training-program]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, docs/loop/tasks/cpu-discrete-writer-core/**, docs/research/notebook/cpu-training.md, docs/start/FILEMAP.md, docs/roadmap/items/cpu-discrete-writer-core.md]
---

## Why it matters

The discrete trainer's selection is split between cell admission and within-cell token ranking. Early runs could improve rank loss while losing admitted targets. A CPU path must expose both before any update is allowed.

## What is missing

Build a new non-default crate that decodes the integer codebook, canonicalizes token vectors, computes exact cell distances, admits the configured kept cells, ranks the full vector, reports top-1/top-8 and admitted-target counts, and breaks ties toward the lower token id. A deterministic benchmark reports rows/s by thread count and checksums.

## Done when

- The crate builds CPU-only with no CUDA feature or silent fallback.
- Finite tests cover malformed tables, cell boundaries, admission ties, lower-id tie-breaking, target admission and top-8.
- Outputs match existing host oracles and at least one stored GPU observation.
- A release benchmark on the Ryzen 9 9950X records rows/s, elapsed time, thread count and a stable checksum.
- One notebook row and a REPORT distinguish component throughput from training or language evidence.

## Cancel if

An existing committed CPU writer already provides exact rank and admission parity at the same semantics. Use it instead of creating a duplicate.

## How to start

Read `crates/uor-r4-discrete/src/trunk.rs`, `address_cells.rs` and `within_cell_writer.rs`, then one stored GPU observation that records top-8 and admitted cells. Write the parity fixture before the CPU implementation. Keep the new behavior behind a non-default crate entry point.

## Rules that apply

- Exact integer arithmetic only for parity; no host shortcut that changes tie order.
- Geometry and table operations only; no softmax or dense matmul.
- A component benchmark is not training or language evidence.

## Sources

- `crates/uor-r4-discrete/src/trunk.rs`
- `crates/uor-r4-discrete/src/address_cells.rs`
- `crates/uor-r4-discrete/src/within_cell_writer.rs`
- Stored Stage T/M1 GPU observations named by the final ORDER.
