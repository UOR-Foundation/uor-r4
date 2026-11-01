# Standalone UOR inverted-file CUDA package design

## Intent

Create a small, independently buildable CUDA C++17 example that demonstrates a 256-cell inverted-file index for codebook-quantized vectors and a build-time PTX admission gate. This is the owner-approved first community package candidate: the CSR bucketed writer plus instruction-inventory enforcement, extracted without a UOR-R4 model or artifact dependency.

## Goals

- Build and update a 256-cell CSR index and probe it with exact integer L1 distance and token-id tie breaking.
- Provide a full-scan control that reproduces the same quota-limited result, not a stronger or different search.
- Enforce a declared four-entry PTX inventory and reject float, tensor, approximate-exponential, missing, extra, and duplicate entries.
- Include a CPU reference, deterministic tests, negative controls, provenance, and an explicit evidence/non-claim record.
- Require only a C++17 compiler for host checks and CUDA Toolkit for AOT compilation/driver execution. No cudarc, UOR-R4 checkout, Python, downloaded model, or third-party data dependency.

## Non-goals and limits

- No general exact kNN claim. Both paths consider only the K admitted cells, so recall is unchanged.
- No language model, model training, API, worker, artifact schema, Claude-owned file, or production deployment change.
- No performance, portability, energy, Faiss-comparison, or cross-library claim without a later registered hardware measurement.
- No copied `s[]` ABI, `forward`, `counterfactual`, `evaluate`, route/gate/trace, or model objective implementation.
- No generated PTX, cubin, binary, model, corpus, or secret committed.

## Public algorithm and interfaces

The fixed codebook has 16 eight-dimensional `int32` root vectors. Token cell = `root_id(first block) * 16 + root_id(last block)`, so 256 cells exist. A non-root endpoint block is an explicit error, never silently mapped to another cell.

C++ host API:

```cpp
enum class Status { Ok, InvalidArgument, NonRootBlock, CsrInvariant, CudaUnavailable, CudaFailure };
struct Index { std::array<uint32_t,257> offsets; std::vector<uint32_t> members; std::vector<uint32_t> cells; uint32_t absent_count; };
struct ScanResult { uint32_t prediction; std::array<uint32_t,16> admitted; std::array<uint32_t,8> top8; };
Status build(const Table&, Index&);
Status update(const Table&, uint32_t moved_token, Index&);
Status probe(const Table&, const Query&, uint32_t k, const Index&, ScanResult&);
Status full_scan(const Table&, const Query&, uint32_t k, ScanResult&);
```

CUDA entries with matching behavior: `uor_if_build`, `uor_if_update`, `uor_if_probe`, `uor_if_full_scan`. A CPU-only build exposes the same host API; the CUDA driver is a separate implementation behind that API, not a duplicate objective.

## Required tests

- PTX admission: clean module accepted; float register/instruction, tensor opcode, approximate exponential, missing, extra, and duplicate entries rejected; comments containing float words accepted.
- Parity: `V ∈ {1,64,256,4096}` and `K ∈ {1,8,16}`, with seeded and single-cell tables, fewer than K occupied cells, and non-root endpoints refused.
- Incremental update: 64 sequential moves compare update against a fresh build after every move; include ABSENT→cell, cell→ABSENT, cell→different cell, and no-op.
- Negative controls: stale bucket, reversed tie, over-prune, and corrupted CSR are each detected; the unmodified control passes.
- Determinism: repeated host tests and two independent AOT builds produce identical output and image hashes where the toolchain supports it.
- No performance benchmark in this unit. GPU timing/portability is `NOT_RUN`.

## Package artifacts

- `projects/uor-cuda-inverted-file/` with `Makefile`, `include/`, `src/`, `tests/`, `README.md`, `LICENSE`, `PROVENANCE.md`, `CITATION.cff`, `CODE_OF_CONDUCT.md`, and `RESULTS.md`.
- `docs/labs/uor-cuda-inverted-file/` with the canonical lab record and generated host results.
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-1/` with ORDER/REPORT for protected delivery.

Generated build output remains ignored and uncommitted. A later Foundation Sandbox repository can copy this directory without rewriting its algorithm or provenance.

## Verification and publication boundary

Pass host tests, admission self-test, AOT compile where nvcc is available, clean-clone build, documentation/claim checks, and fresh exact-head review before merge. Do not submit to NVIDIA or announce performance yet. A later pre-registered GPU run must use a tuned full-scan control, at least two architectures, repeated measurements, and separate kernel-only/end-to-end scopes before any public benchmark or upstream discussion.
