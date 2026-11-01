# UOR inverted-file CUDA example

An exact integer inverted-file index for codebook-quantized vectors, with a CPU oracle and an optional CUDA Driver API implementation. **This is not general exact kNN:** both the bucket probe and full-scan control consider only the `K` admitted cells, so recall is unchanged. Every token's first and last 8-vector must exactly match one of the 16 fixed codebook roots; non-root endpoint blocks are refused rather than silently assigned to another cell.

## Why this is useful

- A 256-cell CSR index with sorted members and incremental cell updates.
- Exact integer L1 distance and token-id tie breaking on CPU and CUDA.
- Four matching CUDA operations: build, update, bucket probe, and full-scan control.
- A build-time PTX admission gate that proves the module contains exactly those four entries and rejects forbidden instructions, tensor operations, approximate exponentials, and inventory drift.
- No UOR-R4 model, artifact schema, cudarc wrapper, Python, downloaded dependency, model checkpoint, corpus, or third-party data requirement.

The transferable idea is the combination of an integer inverted-file index and a mechanical instruction-budget check. The PTX gate also works independently of the UOR geometry.

## Build and test

### CPU-only

```sh
make host-test
```

This builds and runs:

- `admission_test`: 13 admission checks covering clean/commented modules, forbidden instructions, and missing/extra/duplicate entries.
- `inverted_file_test`: seeded parity across `V ∈ {1,64,256,4096}` and `K ∈ {1,8,16}`, single-cell and fewer-than-K cases, non-root refusal, exact ties, 64 incremental updates, removal/reinsertion, stale/corrupt CSR detection, and over-pruning detection.

No GPU, CUDA Toolkit, or UOR-R4 checkout is needed for these tests.

### CUDA AOT build and parity

```sh
# Choose the architecture supported by your toolchain and card
UOR_NVCC_ARCH=compute_80 make cuda
UOR_NVCC_ARCH=compute_80 make cuda-test
```

The CUDA path compiles `src/kernels.cu` to PTX, runs the admission tool, assembles a cubin, loads it through the CUDA Driver API, and compares build/update/probe/full-scan behavior against the CPU oracle. `nvcc`, a compatible NVIDIA driver, and a GPU are required. If any are unavailable, the commands stop with `NOT_RUN` or `CudaUnavailable`; that is not a passing GPU result.

Generated files stay under `build/` and are ignored. `make clean` removes them.

### Optional local telemetry

The standalone CPU example can emit OTLP/HTTP JSON without an SDK or network dependency:

```sh
make instrumented-example                         # default: no telemetry output
UOR_OTEL_OUT=build/trace.json make instrumented-example
make telemetry-test                               # IDs, JSON, default-off, and CPU parity
```

Only an explicit `UOR_OTEL_OUT` path enables the file sink. Each run uses random trace/span IDs; the fixed codebook's SHA-256 digest and 512-byte size stay artifact attributes and are never execution IDs. The build span is the parent of update, probe, and full-scan spans. The sink writes a temporary file and renames it over the current complete in-memory trace after each operation. `TelemetryFailure` reports an export I/O error separately from CUDA failures; CPU errors take precedence and completed CPU side effects are not rolled back.

This instruments the **CPU oracle**, not the CUDA kernels. It does not send data to a collector or emit prompt/completion content. Official Collector Contrib 0.164.0 ingestion and decoded-span verification **PASS** over loopback; SDK/multi-runtime interoperability, GPU timing, and production export are **NOT_RUN**. See [`../../docs/telemetry/uor-conventions.md`](../../docs/telemetry/uor-conventions.md).

## Algorithm and API

The codebook contains 16 roots of eight `int32` values. A token's cell is:

```text
root_id(first 8-vector) * 16 + root_id(last 8-vector)
```

This produces 256 cells. `build` constructs the CSR index. `update` maintains sorted membership and offsets after a token moves cells, including removal to and reinsertion from `ABSENT`. `probe` selects K cells by exact endpoint distance, then ranks only tokens in those cells by full-vector L1 distance and token id. `full_scan` computes the same result independently as a control.

The public C++17 API is in `include/uor_inverted_file.hpp`. CUDA entry points are in `src/kernels.cu`:

- `uor_if_build`
- `uor_if_update`
- `uor_if_probe`
- `uor_if_full_scan`

`src/admission.cpp` strips comments and inspects emitted PTX tokens rather than trusting CUDA source spelling. `src/cuda_driver.cpp` loads only the admitted cubin and resolves exactly those four functions.

## Evidence and non-claims

`RESULTS.md` and `docs/labs/uor-cuda-inverted-file/` distinguish this package's actual checks from the original UOR-R4 bucket-writer's historical results.

Do not infer any of the following without a later registered measurement:

- General exact nearest-neighbor search or changed recall.
- Loss parity for UOR-R4's `ServingPruned` mode.
- End-to-end speedup, occupancy, energy, or a direct-writer ratio for this package.
- Portability beyond a specifically tested architecture.
- A comparison with Faiss or another library.
- Any language-model or useful-conversation capability.

The summary is in `docs/labs/uor-cuda-inverted-file/`; raw manifests and per-run logs are in the linked retained run roots and Hugging Face commits. Final corrected measurements at `719a9a78` pass on RTX 5090/580.178.04 and RTX 4090/580.95.05; earlier attempts are explicitly rejected and not used for claims. Energy, other architectures/drivers, and cross-library comparisons are **NOT_RUN**. The host-only clean-consumer check and corrected independent replication pass at their declared scopes.

## Release preparation and community review

The planned `cuda-inverted-file-v0.1.0` source/evidence archive has [prepared release instructions](RELEASE.md). It will include the clean-consumer host result, two-card/two-driver measurements, independent corrected replication, raw-evidence links, MIT licensing, and source-commit provenance. **Publication and published-asset verification are pending; no public release URL is claimed yet.**

This release is intended for community review, not as an NVIDIA-endorsed sample or the UOR-R4 model's M8 release. NVIDIA/community feedback and any upstream submission remain separate decisions.

See `PROVENANCE.md`, `RESULTS.md`, `CITATION.cff`, and `OUTREACH.md` for provenance, measured results, citation metadata, and outreach drafts.
