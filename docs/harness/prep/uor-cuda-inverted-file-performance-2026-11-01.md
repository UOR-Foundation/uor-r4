# Prep sheet: matched CUDA full-scan performance and replication, 2026-11-01

- Item: #3475. ORDER: `docs/loop/tasks/uor-cuda-inverted-file/kimi-3/ORDER.md`.
- Source: the pushed performance-instrument head named by the non-author FIT and run manifest.
- Question: does an exact parallel-distance full scan preserve every correctness row, and how do bucket probe and that optimized control compare at kernel-only, direct API, and complete-caller scopes on declared hardware?

## Arms

- **Bucket probe:** the existing CSR member walk and exact ranking, with its original one-thread launch.
- **Optimized full scan:** thread 0 computes admission, the block synchronizes, all 256 threads compute full-token L1 distances into bounded dynamic shared memory for count ≤ 4096, the block synchronizes, and thread 0 performs the existing deterministic serial top-8 ordering. Larger counts use thread 0's serial fallback, which is **NOT_RUN** in this unit.
- **Independent oracle:** each timed configuration is checked against the CPU result before warmup. The guarded 504-row CPU/CUDA suite also passes before timing, including exact reinsertion, driver corrupt-CSR refusal, full-vector sensitivity, and extreme values. `oracle_match` and the oracle prediction are emitted per measured row.

## Measurement contract

- V ∈ {64,256,1024,4096}; K ∈ {1,8,16}; operations `probe` and `full_scan`; fixed query seed 7 for every K. Compare each GPU-built index with a fresh CPU build and emit `index_match` before timing.
- Warm up 3 times per configuration, then retain 10 measured repetitions. Do not discard outliers. Raw JSONL has one record per operation/configuration/repetition with all three times, resource fields, status, oracle result, and `index_match`.
- **Kernel-only:** one context and one upload per configuration, with CUDA events around repeated launches. Function/resource/occupancy queries occur outside the timed window; no per-call context creation or copy is included.
- **Direct API:** existing scan wrapper with validation, session/context creation, upload, launch, synchronization, and result readback, using an already built index. No event or resource-query overhead is added to this scope.
- **Complete caller:** the same CPU `uor_inverted_file::build` for both arms plus the public scan path. Full scan uses its existing CPU-built control CSR; probe uses the resulting CSR with `probe`. Both end-to-end results must equal the oracle.
- Record GPU, compute capability, driver release, Toolkit, source, PTX/cubin hashes, registers, static/local/dynamic shared bytes, and max active blocks per SM. Capture `nvidia-smi` and `nvcc --version` into the run root and bind their hashes, the 504-line `correctness.tsv` hash, and `driver_cuda_version`/`toolkit_header_version` separately in the manifest. A manifest binds run id, source, record count, and raw JSONL SHA-256 to the evidence root. No energy or cross-library inference.

## Bars and decisions

- **Correctness bar:** all 504 rows pass and `correctness.tsv` has exactly 504 lines; every benchmark result status is `Ok`; prediction/admitted/top-8 matches the independent oracle for every timed configuration and scope; every GPU-built index equals its fresh CPU build. Any failure stops measurement and is classified as instrument or source defect.
- **Evidence bar:** every configuration has all 10 raw measurements per scope and valid resource fields, plus a complete manifest with correctness and hardware-log hashes. Missing rows, timings, identities, or oracle/index checks are INCOMPLETE, not a ratio.
- **Interpretation:** report median/min/max raw scopes and break-even by V/K. Do not compare against the old spilling baseline or the historical 45.5× model-writer ratio. A ratio is only for the same source/hardware/configuration/scope.
- **Replication:** this unit measures one named card. A second architecture/driver and external replication require separate FITs and run roots and are NOT_RUN here.

## Resources and preservation

One GPU at a time, initially one hour at the displayed Vast rate, within the shared caps. The 504-line evidence file precedes `cuda-benchmark`. Use `make cuda-benchmark UOR_BENCH_RUN_ID=<id> UOR_BENCH_SOURCE=$(git rev-parse HEAD) UOR_BENCH_OUT=/workspace/uor-r4/evals/<id>/benchmark.jsonl UOR_BENCH_CORRECTNESS=/workspace/uor-r4/evals/<id>/correctness.tsv UOR_BENCH_GPU_INFO=/workspace/uor-r4/evals/<id>/gpu-info.txt UOR_BENCH_NVCC_INFO=/workspace/uor-r4/evals/<id>/nvcc-info.txt`, after creating `<id>` as a new run root. Command-line variables record but do not change the head. Preserve every raw JSONL, manifest, module, command, source identity, failure, and retry in that root before teardown. Upload to the project store and verify read-back. No generated binary is committed.

TEST FITNESS: PENDING — a non-author FIT is required after the instrument is written and before any GPU measurement. No timing has run.

## FIT record

**TEST FITNESS: FIT**, non-author, 2026-11-01. Exact source head: `7badaf6ccd18fd4ec8768a197b84347c2f138795`. Scope: one RTX 5090, compute capability 12.0, one driver release and one CUDA 12.8 toolkit captured in `gpu-info.txt` and `nvcc-info.txt`; V ∈ {64,256,1024,4096}, K ∈ {1,8,16}, fixed query seed 7, warmup 3 plus 10 retained repetitions, kernel-only/direct-API/complete-caller scopes, 240 JSONL records and a 504-row `correctness.tsv`. The manifest's `source` must equal this head. The >4096 serial fallback, other V/K, second architecture/driver, external replication, concurrent kernels, energy, cross-library comparison, and release claims are NOT_RUN. Any change to kernel, driver, correctness/benchmark tests, or Make recipe requires a new pushed head and FIT.

## Attempt 1 and retry

Attempt 1 at `5bc7c547e` stopped at C++ compilation before correctness or timing because the complete-caller `probe` call was ambiguous between CPU and CUDA APIs. The log was archived at `local/kimi/cuda-performance-1/`, the pod was deleted, and no measurement was produced. Retry head `3fd8408f20c6be329aed683b689011e6443dacca` qualifies that call as `uor_inverted_file::cuda::probe` and has a fresh FIT and `cuda-performance-2` evidence root.

## Attempt 2 and retry

Attempt 2 at `3fd8408f2` passed all 504 correctness rows but stopped before timing because the file-hash helper treated normal EOF as an error. Logs and the 504-row file were archived at `local/kimi/cuda-performance-2/`, the pod was deleted, and no measurement was produced. Retry head `7badaf6ccd18fd4ec8768a197b84347c2f138795` changes that check to `input.bad()` and has a fresh FIT and `cuda-performance-3` evidence root.
