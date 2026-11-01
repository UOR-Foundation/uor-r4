# Verification results

Date: 2026-11-01. Source base: `7d8a0c2642600f5ff02d5ef30d130ec8fe8056d8`. Attempt 1 (`71c84dfa3`) failed C++ compilation and produced no parity result; the passing attempt-2 source was `aef414859`, with a later guard/evidence repair recorded separately.

## Host and clean-copy results

`make evidence` runs the host suite twice, creates a clean copy containing only `projects/uor-cuda-inverted-file/`, runs it there, and compares all three outputs:

```sh
make evidence
```

All three runs produced this exact output:

```text
admission tests: 13/13 passed
inverted-file tests: all parity, update, refusal, and fault checks passed
```

Normalized test-output SHA-256 for each run:

```text
7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d
```

The clean copy was created in a fresh directory, and no UOR-R4 source path or checkout was referenced by the build. The byte-identical outputs demonstrate repeatability of the test text, not independent GPU or cross-architecture correctness.

## Covered behavior

- Clean and comment-only PTX acceptance; float/tensor/approximate-exponential rejection; missing/extra/duplicate entry rejection.
- Seeded parity for `V ∈ {1,64,256,4096}` and `K ∈ {1,8,16}`, with all 32 vector elements varied; a 24-row sensitivity check requires at least three top-8 changes and one prediction change against the zero-middle control.
- Single-cell collapse, fewer than K occupied cells, non-root endpoint refusal, and exact token-id tie breaking.
- Sixty-four incremental updates with fresh-build CSR equality and search parity after each update.
- Removal to and reinsertion from `ABSENT`.
- Stale-cell, corrupted-offset, over-pruned-result, and reversed-tie controls, with the unmodified control passing.

## Single-card CUDA correctness

At source `aef414859cf289c7b35883da4f60f07bcb1d95c1`, one RTX 5090 (compute capability 12.0, driver 580.178.04) with CUDA 12.8.93 passed AOT compilation, four-entry admission, planted-entry rejection, and Driver API execution. All three row-producing executions passed **498/498 rows**, and the three files were byte-identical (**1,494 passing rows total**). Clean-rebuild PTX and cubin hashes were identical. Evidence is in `docs/labs/uor-cuda-inverted-file/gpu-sm120-results.json` and `docs/loop/tasks/uor-cuda-inverted-file/kimi-2/REPORT.md`.

`cuobjdump -arch` produced an empty file and is not cited as architecture proof. Full scan used a CPU-built control CSR; probe used the GPU-built CSR.

## NOT_RUN

The laptop still has no `nvcc`; the successful CUDA result came from a leased RTX 5090, not the laptop. The following remain **NOT_RUN**, not passes:

- GPU timing, tuned/end-to-end speedup, occupancy, energy, and comparison counts.
- Additional architectures, driver versions, independent hardware replication, and external consumers.
- Stale/corrupt CUDA update rows, all-absent CUDA scans, K=0/17, V=1/4096, and concurrent kernels.
- Faiss or any other cross-library comparison.

The original UOR-R4 bucket-writer's historical results are provenance only and are not attributed to this package.
