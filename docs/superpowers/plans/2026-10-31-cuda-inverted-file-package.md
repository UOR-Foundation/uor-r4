# Standalone UOR Inverted-File Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an independently verifiable CUDA C++17 CSR inverted-file example with a CPU oracle and build-time PTX admission enforcement.

**Architecture:** One small host API fronts a CPU reference and an optional CUDA Driver API implementation. The 256-cell index, exact quota-limited probe, full-scan control, and four-entry PTX admission gate have no UOR-R4 model, artifact, or third-party library dependency.

**Tech Stack:** C++17, CUDA C++, CUDA Driver API, Make, deterministic host tests.

**Spec:** `docs/superpowers/specs/2026-10-31-cuda-inverted-file-package-design.md`

## Global Constraints

- No edits to `crates/**`, model artifacts, API/worker/engine code, Claude paths, or `.github/**`.
- No general exact-kNN, performance, portability, energy, Faiss-comparison, or language-capability claim.
- No Python, downloaded dependency, committed binary/PTX/cubin/model/corpus/secret, or production deployment.
- GPU timing and additional-architecture measurements remain `NOT_RUN` in this unit.
- Host tests must pass before AOT compilation; all four CUDA entry points must pass admission exactly.

## Review Focus

- Non-root endpoint blocks: return `NonRootBlock` and increment absent/error state rather than aliasing a cell (Tasks 2–3).
- ABSENT/cell/no-op moves: preserve CSR offsets, sorted members, and absent count exactly (Task 2).
- K greater than occupied cells and single-cell collapse: return only existing cells/tokens, never ABSENT as a successful prediction (Task 2).
- Float words in comments versus executable instructions: comments pass; real forbidden instructions fail (Task 1).
- Full-scan baseline must not silently become a different search: probe and full scan return identical predictions/admitted cells/top8 (Tasks 2–3).

---

### Task 1: Package skeleton and PTX admission gate

**Files:**
- Create: `projects/uor-cuda-inverted-file/Makefile`
- Create: `projects/uor-cuda-inverted-file/src/admission.hpp`
- Create: `projects/uor-cuda-inverted-file/src/admission.cpp`
- Create: `projects/uor-cuda-inverted-file/tests/admission_test.cpp`
- Create: `projects/uor-cuda-inverted-file/.gitignore`
- Create: `projects/uor-cuda-inverted-file/LICENSE`
- Create: `projects/uor-cuda-inverted-file/PROVENANCE.md`

**Interfaces:**
- Produces: `Status admit_ptx(std::string_view ptx, std::span<const std::string_view> entries)`; forbidden type/instruction inventory and exact four CUDA entries used by later tasks.
- Consumes: nothing outside C++17 standard library.

- [ ] **Step 1: Write failing admission tests**
  Cover clean four-entry PTX, float type/instruction, tensor opcode, approximate exponential, missing, extra, duplicate entries, and float words only in comments.
- [ ] **Step 2: Compile and run tests to verify failure**
  Run: `make -C projects/uor-cuda-inverted-file host-test`.
  Expected: compilation fails because admission API is absent.
- [ ] **Step 3: Implement tokenizer and exact-inventory admission**
  Strip comments before tokenization; reject `f16/f16x2/f32/f64/bf16/bf16x2/tf32/e4m3/e5m2/mma/wmma/wgmma/tcgen05/tensor/tensormap` components and `ex2.approx` prefixes. Require exactly the four entries, each once.
- [ ] **Step 4: Run admission tests to verify pass**
  Run: `make -C projects/uor-cuda-inverted-file host-test`.
  Expected: all admission tests pass with zero failed assertions.
- [ ] **Step 5: Add provenance and ignored build outputs**
  Name source commit `3d39b0c720dbe95693efe8aea93b3873247a7ba1`, original bucket/build files, extraction boundaries, MIT license, and no NVIDIA sample-source reuse. Ignore `build/`, `dist/`, `*.ptx`, `*.cubin`, and binaries.
- [ ] **Step 6: Commit**

### Task 2: CPU reference CSR index and parity controls

**Files:**
- Create: `projects/uor-cuda-inverted-file/include/uor_inverted_file.hpp`
- Create: `projects/uor-cuda-inverted-file/src/inverted_file.cpp`
- Create: `projects/uor-cuda-inverted-file/src/codebook.cpp`
- Create: `projects/uor-cuda-inverted-file/tests/inverted_file_test.cpp`

**Interfaces:**
- Produces: the spec's `Status`, `Index`, `ScanResult`, `build`, `update`, `probe`, `full_scan`, deterministic table/query fixtures, and CSR invariant validator.
- Consumes: Task 1's `Status`/error taxonomy or maps it explicitly to the same values.

- [ ] **Step 1: Write failing parity and refusal tests**
  Cover V=1/64/256/4096, K=1/8/16, single-cell collapse, fewer than K occupied cells, non-root endpoints, 64 sequential moves, four planted faults plus unmodified control, and exact token-id tie breaking.
- [ ] **Step 2: Run tests to verify failure**
  Run: `make -C projects/uor-cuda-inverted-file host-test`.
  Expected: link/compile failure because CPU index/search is absent.
- [ ] **Step 3: Implement fixed codebook, CSR build/update, probe, and full-scan control**
  Use exact L1 distance and token-id tie break. Return `NonRootBlock` for non-root endpoints. Preserve sorted members/offsets/absent count after every move. Both search paths consider only K admitted cells.
- [ ] **Step 4: Run all host tests to verify pass**
  Run: `make -C projects/uor-cuda-inverted-file host-test`.
  Expected: all CPU parity/refusal tests pass, including fresh-build equality after each of 64 moves and detection of every planted fault.
- [ ] **Step 5: Commit**

### Task 3: AOT CUDA module and driver implementation

**Files:**
- Create: `projects/uor-cuda-inverted-file/src/kernels.cu`
- Create: `projects/uor-cuda-inverted-file/src/cuda_driver.hpp`
- Create: `projects/uor-cuda-inverted-file/src/cuda_driver.cpp`
- Create: `projects/uor-cuda-inverted-file/tests/cuda_parity_test.cpp`
- Modify: `projects/uor-cuda-inverted-file/Makefile`

**Interfaces:**
- Produces: `uor_if_build`, `uor_if_update`, `uor_if_probe`, `uor_if_full_scan`; optional CUDA-backed implementation of the Task 2 API and typed `CudaUnavailable` result when no driver/device exists.
- Consumes: Task 2 fixtures/expected outputs and Task 1 admission function.

- [ ] **Step 1: Add CUDA parity test entry point and confirm no default GPU is required for host tests**
  `make host-test` must remain CPU-only. CUDA parity is a separate explicit target and reports `CudaUnavailable`, never a silent pass.
- [ ] **Step 2: Implement the four CUDA entries with exact CPU behavior**
  Mirror CSR build/update and quota-limited probe/full-scan. Do not add routes, model objective, floats, tensor operations, or a different search.
- [ ] **Step 3: Implement minimal CUDA Driver API loader**
  Load only the admitted cubin; resolve exactly four functions; validate buffers/status; distinguish unavailable driver/device from kernel/output failure.
- [ ] **Step 4: Compile and admit the module where nvcc is available**
  Run: `make -C projects/uor-cuda-inverted-file cuda`.
  Expected: PTX and cubin build, admission passes, and no generated artifact is tracked. If nvcc is unavailable, record `NOT_RUN` rather than claiming compilation passed.
- [ ] **Step 5: Run CUDA parity when a GPU is available**
  Run: `make -C projects/uor-cuda-inverted-file cuda-test`.
  Expected: all CPU/GPU parity tests pass; otherwise record `CudaUnavailable` with no claim of GPU execution.
- [ ] **Step 6: Commit**

### Task 4: Evidence, clean-clone verification, and delivery

**Files:**
- Create: `projects/uor-cuda-inverted-file/README.md`
- Create: `projects/uor-cuda-inverted-file/CITATION.cff`
- Create: `projects/uor-cuda-inverted-file/CODE_OF_CONDUCT.md`
- Create: `projects/uor-cuda-inverted-file/RESULTS.md`
- Create: `docs/labs/uor-cuda-inverted-file/README.md`
- Create: `docs/loop/tasks/uor-cuda-inverted-file/kimi-1/ORDER.md`
- Create: `docs/loop/tasks/uor-cuda-inverted-file/kimi-1/REPORT.md`

**Interfaces:**
- Produces: reproducible quickstart, explicit non-claim list, generated host evidence, and protected-delivery records.

- [ ] **Step 1: Document exact scope and every non-claim**
  First paragraph states the codebook-root constraint and quota-limited search. List NOT_RUN GPU timing, portability, energy, and cross-library comparisons. Distinguish this new package's host evidence from historical bucket-writer records.
- [ ] **Step 2: Generate host results and clean-clone build**
  Run host tests twice and compare result hashes. Copy the package alone to a clean temporary directory and rerun `make host-test`; no UOR-R4 path or checkout may be referenced.
- [ ] **Step 3: Run documentation and scope checks**
  Run claim-wording, `git diff --check`, and confirm no `crates/**`, model, Claude, API/worker/engine, generated binary, or `.github/**` change.
- [ ] **Step 4: Obtain fresh exact-head review and deliver through protected PR**
  Include CPU/admission evidence and every NOT_RUN. Do not open NVIDIA issues, publish a release, or announce a benchmark in this unit.
- [ ] **Step 5: Commit**

## Self-review

- Spec coverage: Tasks 1–4 implement admission, CPU oracle, CUDA implementation, negative controls, provenance, and delivery evidence without a benchmark or model dependency.
- Type consistency: `Status`, `Index`, `ScanResult`, and the four CUDA entry names match across host and device tasks.
- Review focus: every named failure class has a test or an explicit NOT_RUN with a loud unavailable result.
- Proportion: tasks carry implementation/test decisions, not transcripts of kernel bodies. The implementer writes those bodies against the listed interfaces.
