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

- `admission_test`: 10 admission checks covering clean/commented modules, forbidden instructions, and missing/extra/duplicate entries.
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

GPU timing, additional architectures, energy, and cross-library comparisons are **NOT_RUN** for this package.

## Before a public release

- Build from a clean clone and run the full host suite.
- Compile and run CUDA parity on supported hardware, including negative controls and repeated determinism checks.
- Measure against a tuned full-scan control on at least two architectures, separating kernel-only from end-to-end results.
- Publish raw evidence and exact scope rather than extrapolating the original bucket-writer's results.
- Keep MIT licensing and source-commit provenance. A Foundation Sandbox application and NVIDIA/community feedback should follow only after an external consumer can build and use the example.

See `PROVENANCE.md`, `RESULTS.md`, and `CITATION.cff`.
