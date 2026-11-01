# Verification results

Date: 2026-11-01. Source base: `3d39b0c720dbe95693efe8aea93b3873247a7ba1`.

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
- Seeded parity for `V ∈ {1,64,256,4096}` and `K ∈ {1,8,16}`.
- Single-cell collapse, fewer than K occupied cells, non-root endpoint refusal, and exact token-id tie breaking.
- Sixty-four incremental updates with fresh-build CSR equality and search parity after each update.
- Removal to and reinsertion from `ABSENT`.
- Stale-cell, corrupted-offset, over-pruned-result, and reversed-tie controls, with the unmodified control passing.

## NOT_RUN

`nvcc` was unavailable on the verification machine. Both `make cuda` and `make cuda-test` stopped with:

```text
NOT_RUN: nvcc is unavailable; CUDA compilation and tests were not performed
```

Therefore the following are **NOT_RUN**, not passes:

- AOT CUDA compilation and admission on emitted module bytes.
- CUDA Driver API execution and CPU/GPU parity.
- GPU timing, end-to-end speedup, occupancy, energy, and comparison counts.
- Additional architectures, driver versions, and independent hardware replication.
- Faiss or any other cross-library comparison.

The original UOR-R4 bucket-writer's historical results are provenance only and are not attributed to this package.
