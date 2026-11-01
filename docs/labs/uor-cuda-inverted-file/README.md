# Standalone UOR inverted-file CUDA package — host verification

Date: 2026-11-01. Lab: kimi. Item: `uor-cuda-inverted-file` (#3475). Base: `3d39b0c720dbe95693efe8aea93b3873247a7ba1`.

This record covers a new standalone CUDA C++17 package in `projects/uor-cuda-inverted-file`. It is not a model evaluation or a repeat of the original bucket-writer GPU qualification.

## Result

- PTX admission: **13/13 checks passed**.
- CPU reference and full-scan parity: **PASS** for seeded tables, `V ∈ {1,64,256,4096}`, `K ∈ {1,8,16}`.
- Incremental update: **64/64 moves** with fresh-build CSR equality and search parity after each move.
- Removal/reinsertion from `ABSENT`, non-root refusal, exact ties, and planted stale/corrupt/over-pruned controls: **PASS**.
- Two repeated host runs and one clean-copy run produced identical test output, SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`.
- Clean-copy host build required no UOR-R4 checkout or external dependency.

[Machine-readable host results](host-results.json) and [package results](../../../projects/uor-cuda-inverted-file/RESULTS.md) carry the exact commands and limitations.

## NOT_RUN

`nvcc` was unavailable. AOT CUDA compilation and CUDA Driver API execution are **NOT_RUN**, so this record makes no GPU correctness, performance, portability, energy, or cross-library claim. GPU timing and additional-architecture evidence require a separate pre-registered run with a tuned full-scan control.

## Provenance

The extraction is based on `crates/uor-r4-discrete/src/bucket_source.rs`, `build.rs`, and the generated fixed codebook from UOR-R4's geometry build. Historical bucket-writer records are not treated as evidence for this new package. No NVIDIA sample source is copied. Package license: MIT.
