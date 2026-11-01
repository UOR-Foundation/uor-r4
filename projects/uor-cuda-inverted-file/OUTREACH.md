# CUDA package release and outreach drafts

Status: **published as [`cuda-inverted-file-v0.1.0`](https://github.com/UOR-Foundation/uor-r4/releases/tag/cuda-inverted-file-v0.1.0), with downloaded assets checksum- and extracted-test-verified. Not submitted and not endorsed by NVIDIA.** The measured implementation is `719a9a7861deb595dfc5269ffec8a5e7e53cb39b`. Earlier rejected attempts are excluded from every claim below.

## Release summary

An exact integer inverted-file index for codebook-quantized vectors, with a CPU oracle and optional CUDA Driver API implementation. The useful transferable pieces are a 256-cell CSR index with sorted incremental updates, exact integer L1 distance and token-id tie breaking, and a build-time PTX admission gate that accepts exactly four entries and rejects forbidden instructions or inventory drift.

This is **not general exact kNN**: bucket probe and full scan consider only the K admitted cells, preserving recall. It is not a UOR-R4 model, checkpoint, or language-capability claim.

## Verified evidence

- Host and clean-copy checks: 13/13 admission checks plus all host parity/update/refusal/fault checks. External clean-consumer hashes match the final source.
- AOT and Driver API correctness on RTX 5090: 504/504 rows in each of three byte-identical guarded executions, with identical clean-rebuild module hashes.
- Matched measurements on RTX 5090/580.178.04 and RTX 4090/580.95.05: 504 correctness rows and 240 measurements per card, with per-launch oracle checks, complete-caller warmups, and corrected complete-caller timing boundaries.
- Independent corrected replication on another RTX 5090/595.71.05 with the same source, 504 rows, 240 measurements, and verified source/module/hash manifests.
- Local and Hugging Face read-back verification for final evidence; all measurement pods deleted after archival.

### Scope-limited results

| Card / driver | Operation | Kernel median (ms) | Direct API median (ms) | Complete-caller median (ms) |
|---|---|---:|---:|---:|
| RTX 5090, 580.178.04 | full_scan | 0.109856 | 115.493 | 115.407 |
| RTX 5090 | probe | 0.160144 | 115.678 | 115.620 |
| RTX 4090, 580.95.05 | full_scan | 0.114944 | 188.424 | 188.272 |
| RTX 4090 | probe | 0.190448 | 188.447 | 188.839 |

Probe/full kernel-median ratios by V are approximately 1.09×, 1.69×, 1.76×, and 1.81× on the 5090, and 1.13×, 1.73×, 1.98×, and 1.90× on the 4090 at V = 64, 256, 1024, and 4096. These are same-source, same-card, same-configuration **kernel-only** ratios, not cross-card or end-to-end speedups. Wrapper overhead dominates direct and complete-caller scopes.

Evidence:

- RTX 5090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/acc77e6f8b7f290deb4bab28a549be4083f3c9c4
- RTX 4090: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/89e10d8b248383b152f2db81cd421e000b7bdf10
- Independent corrected replication: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/dddfebf8a5ccd006a5f9e0045e1e46b2feeacb82

## Build and test

```sh
make host-test
make telemetry-test

# On a supported GPU, choose the appropriate architecture:
UOR_NVCC_ARCH=compute_120 make cuda
UOR_NVCC_ARCH=compute_120 make cuda-test
make cuda-benchmark UOR_BENCH_RUN_ID=my-run UOR_BENCH_SOURCE=$(git rev-parse HEAD)
```

`nvcc`, a compatible NVIDIA driver, and a supported GPU are required for the optional CUDA path. `make evidence` checks repeated and clean-copy host behavior. Telemetry remains opt-in via `UOR_OTEL_OUT`; no telemetry runs by default.

## Draft cuda-samples clarification issue

**Title:** Standalone integer inverted-file index with a PTX instruction/inventory admission gate

**Question for maintainers:** Would a small, self-contained example of an exact integer inverted-file index and its PTX admission tooling fit the scope and contribution process of `cuda-samples`? The example uses the CUDA Driver API, no third-party model/data dependency, exact integer L1 distance and deterministic tie breaking, and a gate that accepts exactly four entries and rejects forbidden instructions or inventory drift.

**Why it may be useful:** it demonstrates incremental CSR updates, exact CPU/GPU parity, and scoped comparisons against a tuned full-scan control rather than a strawman. The report includes raw evidence on two architectures and driver releases plus an independent corrected replication. I am asking for fit and feedback, not endorsing a performance claim or requesting an architecture PR.

**Links:** final source and raw evidence above. `RESULTS.md`, `gpu-sm120-results.json`, `gpu-performance-results.json`, and the ORDER/REPORT in the repository contain the exact scopes and rejected attempts.

## Draft Developer Forums post

**Title:** An exact integer inverted-file CUDA example with a mechanical PTX admission gate

I built a small standalone example combining a 256-cell CSR index, incremental updates, exact integer L1 distance, and a PTX instruction/inventory admission gate. The CPU and CUDA implementations agree on all 504 guarded correctness rows, including updates, reinsertion, non-root refusal, and extreme values. An independent rerun reproduced the same source and record counts on a third driver release.

The full-scan comparator uses parallel distance evaluation with deterministic serial top-8 ordering. On the declared grid, its median kernel time is lower than the bucket probe at each measured size on both cards. I also report direct API and complete-caller times because wrapper overhead dominates those scopes; the kernel-only ratios are not end-to-end speedups.

**Technical question:** is the PTX admission approach useful as an example of constraining a small integer kernel's public entry inventory and instruction set, and would the current build/benchmark packaging make a suitable standalone sample? Raw manifests, module hashes, negative controls, and exact timing scopes are linked above. Feedback on correctness and measurement methodology is welcome.

## Not claimed

- General exact nearest-neighbor search, changed recall, or `ServingPruned` loss parity.
- End-to-end speedup, portability beyond the named cards/grid, energy, or a cross-library comparison.
- NVIDIA acceptance, production deployment, or any language-model capability.

The source/evidence release is public, and its downloaded assets passed checksum and extracted-test verification. Opening the clarification issue or forum thread remains a separate owner decision; a Foundation Sandbox application is optional, not an NVIDIA endorsement or a required next step.
