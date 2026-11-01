# Standalone CUDA Public Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task with TDD and a final whole-branch review.

**Goal:** Produce hardware-verified correctness, matched performance evidence, and external-consumer evidence sufficient to decide whether the standalone CUDA package is ready for public presentation.

**Architecture:** Keep the standalone C++17/CUDA package independent of the model. Gate progression on real emitted-module correctness, then add a matched optimized full-scan comparator and separate timing scopes, and only then replicate on additional hardware and through an external consumer.

**Tech Stack:** CUDA C++17, CUDA Driver API, Make, C++ host tests, `scripts/pod/uor-pod`, JSON evidence.

**Spec:** `projects/uor-cuda-inverted-file/ADVERTISING.md` and `docs/harness/prep/uor-cuda-inverted-file-sm120-2026-11-01.md`.

## Global Constraints

- No model, artifact, API, worker, engine, desktop, `.github/**`, Claude-owned, or production-deployment changes.
- GPU work uses only `scripts/pod/uor-pod` after non-author FIT, within ≤4 pods and ≤$8/h across all labs.
- No general-kNN, changed-recall, energy, cross-library, portability, or performance claim without its own measured scope.
- Preserve every negative, command, raw log, source/module identity, retry, cost, and NOT_RUN. No generated binaries in git.
- A correctness pass does not imply tuned performance or release readiness.

## Review Focus

- AOT text may hide unsupported instructions or inventory drift: inspect actual emitted PTX and test negative admission controls (Task 1).
- CUDA may load but compute incorrect CSR/state: compare all operations and faults against fresh CPU behavior (Task 1).
- A slow serial full scan is not a tuned comparator: preserve exact outputs and matched arithmetic/candidate/transfer rules (Task 2).
- Timing boundaries may hide setup/transfer costs: report kernel-only, direct API, and end-to-end scopes separately (Task 3).
- One architecture/driver or the author's checkout is insufficient: repeat on named additional hardware and a clean external consumer (Task 4).

---

### Task 1: RTX 5090 AOT and correctness gate

**Files:** `docs/harness/prep/uor-cuda-inverted-file-sm120-2026-11-01.md`, `docs/loop/tasks/uor-cuda-inverted-file/kimi-2/*`, `docs/labs/uor-cuda-inverted-file/*`, and any narrowly scoped package repair exposed by actual output.

- [ ] Obtain non-author FIT before leasing; run host tests and record the exact pushed source.
- [ ] On one leased RTX 5090, run clean `UOR_NVCC_ARCH=compute_120 make cuda` and `make cuda-test` three times, preserving all output and module hashes.
- [ ] Classify any instrument or source failure; stop before performance work. Commit evidence and the actual NOT_RUN set.

### Task 2: Matched optimized full-scan control

**Files:** standalone package source/tests and a new pre-registration for GPU measurements.

- [ ] Add host correctness tests proving the optimized control matches the existing independent full scan and bucket probe on ties, sparse/dense cells, K limits, updates, and extreme values.
- [ ] Implement the smallest lawful CUDA optimization with identical integer arithmetic, candidate admission, output order, and four-entry inventory; keep PTX admission green.
- [ ] Benchmark only after a fresh FIT and repeat parity/determinism on the target GPU.

### Task 3: Scoped performance evidence

**Files:** package benchmark target, raw JSON/log evidence, `RESULTS.md`, `ADVERTISING.md`.

- [ ] Pre-register sizes, K values, warmups, repetitions, matched transfer rules, and resource measurements.
- [ ] Measure kernel-only, direct API/writer, and complete caller scopes separately; retain every raw run and compiler/resource report.
- [ ] Report distributions/ranges and break-even regions, not only best-case ratios. Do not reuse historical model-writer numbers as package results.

### Task 4: Portability, replication, and release handoff

**Files:** hardware matrix evidence, external-consumer record, README/release notes, ORDER/REPORT.

- [ ] Repeat correctness and measurements on a second named architecture and second driver version, or record precisely what remains blocked.
- [ ] Have a non-author repeat the declared checks and an external consumer build/copy only the package from a clean location.
- [ ] Update release/non-claim docs, run complete host and fresh exact-head review, deliver through protected PRs, and verify on fresh main.
- [ ] Only after all required gates pass, prepare—but do not automatically publish—a cuda-samples clarification issue, Developer Forums evidence post, and optional Foundation Sandbox application.
