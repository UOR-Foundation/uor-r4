# Standalone UOR inverted-file CUDA package — host verification

Date: 2026-11-01. Lab: kimi. Item: `uor-cuda-inverted-file` (#3475). Base: `7d8a0c2642600f5ff02d5ef30d130ec8fe8056d8`. Attempt 1 (`71c84dfa3`) failed C++ compilation and produced no parity result; the passing attempt-2 source was `aef414859`, with a later guard/evidence repair recorded separately.

This record covers a new standalone CUDA C++17 package in `projects/uor-cuda-inverted-file`. It is not a model evaluation or a repeat of the original bucket-writer GPU qualification.

## Result

- PTX admission: **13/13 checks passed**.
- CPU reference and full-scan parity: **PASS** for seeded tables, `V ∈ {1,64,256,4096}`, `K ∈ {1,8,16}`.
- Incremental update: **64/64 moves** with fresh-build CSR equality and search parity after each move.
- Removal/reinsertion from `ABSENT`, non-root refusal, exact ties, and planted stale/corrupt/over-pruned controls: **PASS**.
- Two repeated host runs and one clean-copy run produced identical test output, SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`.
- Clean-copy host build required no UOR-R4 checkout or external dependency.
- Separately, one RTX 5090 passed AOT/CUDA correctness at `18a1c8ad7`: 504/504 rows in each of three byte-identical executions, with identical module hashes after a clean rebuild. See [GPU results](gpu-sm120-results.json).
- Final corrected measurements at `719a9a78` pass on RTX 5090/580.178.04 and RTX 4090/580.95.05: 504 correctness rows and 240 records per card, with all launch/oracle/index checks passing. See [performance results](gpu-performance-results.json). The independent corrected RTX 5090/595.71.05 replication and host-only clean-consumer check also passed.

[Machine-readable host results](host-results.json) and [package results](../../../projects/uor-cuda-inverted-file/RESULTS.md) carry the exact commands and limitations.

## NOT_RUN

On the laptop, `nvcc` is unavailable, so local AOT/CUDA execution is **NOT_RUN**. The separate RTX 5090 correctness result above is limited to its named source/card/grid. Any replication beyond the completed independent RTX 5090/595.71.05 check, additional architectures/drivers, energy, and cross-library comparison remain **NOT_RUN**.

## Provenance

The extraction is based on `crates/uor-r4-discrete/src/bucket_source.rs`, `build.rs`, and the generated fixed codebook from UOR-R4's geometry build. Historical bucket-writer records are not treated as evidence for this new package. No NVIDIA sample source is copied. Package license: MIT.
