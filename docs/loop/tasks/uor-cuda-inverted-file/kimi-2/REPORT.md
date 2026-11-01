# REPORT: standalone CUDA hardware correctness, kimi-2

Lab: kimi. Item: #3475. Order: [ORDER.md](ORDER.md). FIT-approved retry source: `aef414859cf289c7b35883da4f60f07bcb1d95c1`.

## Negatives and limits first

- The first non-author FIT was NOT_FIT: ABSENT→cell was unreachable in CUDA, and the fixture's zeroed middle blocks could not test full-vector ranking. The second FIT found that upload length was incorrectly reused for readback. Repairs added meaningful 32-element fixtures, sensitivity checks, explicit transfer lengths, and exact fresh-build reinsertion comparisons before any lease.
- Attempt 1 at `71c84dfa3` passed host/AOT/admission but failed C++ compilation on three const kernel-count addresses. No CUDA parity ran. The failure was archived and `vast-58641884` was deleted. Three declarations became mutable under a new FIT.
- The retry runbook initially retained two attempt-1 evidence paths. These were corrected before leasing. Attempt 1 remains sealed at `local/kimi/cuda-release-1/`; attempt 2 uses `local/kimi/cuda-release-2/`.
- `cuobjdump -arch` produced an empty file. It is **not evidence** of the architecture. The actual evidence is the recorded `compute_120`/`sm_120` build flags and successful execution on the named RTX 5090.
- No performance, tuned-comparator, second architecture/driver, energy, cross-library, independent-replication, external-consumer, or release result was produced. Stale/corrupt CUDA update rows, all-absent CUDA scans, K=0/17, V=1/4096, and concurrent kernels remain NOT_RUN. CPU and CUDA are separate transcriptions, so shared design errors remain outside parity detection.

## Measured result

On `vast-58643094`, one NVIDIA GeForce RTX 5090 (compute capability 12.0, driver 580.178.04) with CUDA 12.8.93:

- Clean host oracle: 13/13 admission checks and all host parity/refusal/fault checks passed.
- AOT compilation: PASS. Emitted PTX admission: exactly four entries. A planted fifth entry was rejected.
- CUDA Driver API smoke: all build/update/probe/full-scan checks passed.
- Three `--out` executions: **498/498 rows passed in each; 1,494 total rows; all three files byte-identical**.
- A clean rebuild produced identical PTX and cubin hashes. PTX SHA-256: `4912c3b3e082db00ae91f3f5ad6394970f758982ada15adc2cd4a56b2d8bfbc1`. Cubin SHA-256: `d9a838d3b1267ed02e320a3c72372de6eb8481d212a23e95661cd591c3374b04`.
- Reinsertion matched a fresh CPU build exactly. Probe used the GPU-built CSR; full scan intentionally used a CPU-built control CSR.
- Evidence was pulled with SHA-256 verification to `local/kimi/cuda-release-2/`, uploaded to the [project store](https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/f285a26e4b2b14e04df8432f0f6e90594de491dc), and verified again after download. The lease was released and the pod deleted. Claude's separate pod was not touched.

The two leases were each quoted at $0.563/h. Provider invoice totals were not collected, so no exact dollar spend is claimed. Machine-readable results are in `docs/labs/uor-cuda-inverted-file/gpu-sm120-results.json`.

## Decision

The declared single-card correctness gate passes. This unlocks only the separately reviewed matched full-scan performance instrument and additional-architecture work. It is not a performance, portability, production, or public-release qualification. The package's advertising hold remains in force.
