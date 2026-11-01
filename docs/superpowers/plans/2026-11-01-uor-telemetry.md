# Opt-in UOR Telemetry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with TDD and a final whole-branch review.

**Goal:** Add an opt-in, local-only OTLP/HTTP JSON telemetry consumer to the standalone inverted-file example.

**Architecture:** A small C++17 SHA-256 and telemetry module feeds an explicit file exporter. Existing index behavior stays unchanged; an instrumented wrapper emits resource/span attributes with random execution IDs and artifact digests as metadata.

**Tech Stack:** C++17, Make, OTLP/HTTP JSON, standard-library-only tests.

**Spec:** `docs/superpowers/specs/2026-11-01-uor-telemetry-design.md`

## Global Constraints

- No model, API, worker, engine, Claude-owned, desktop, `.github/**`, or deployment changes.
- No default endpoint, telemetry, socket, prompt/completion text, user-turn hash, third-party dependency, or secret.
- No content-derived execution IDs, general-kNN, performance, portability, energy, or upstream-acceptance claim.
- GPU timing remains NOT_RUN.

## Review Focus

- Default disabled must create no output file or endpoint (Tasks 1–2).
- IDs must be 32/16 lowercase hex and unique per root/span; digests never become IDs (Task 1).
- JSON must be valid OTLP/HTTP shape with correct parent IDs and no prompt fields (Task 2).
- Artifact identity stays stable across runs while execution IDs differ (Task 3).
- Existing host/evidence tests must not depend on telemetry (Task 3).

---

### Task 1: SHA-256 and execution identifiers

**Files:** `src/sha256.hpp`, `src/sha256.cpp`, `src/telemetry_ids.hpp`, `src/telemetry_ids.cpp`, `tests/telemetry_ids_test.cpp`

- [ ] Write failing known-vector and ID tests: empty/abc SHA-256, 32/16 lowercase hex, nonzero and unique IDs, no digest-derived IDs.
- [ ] Run `make telemetry-test` and watch compilation fail for missing implementation.
- [ ] Implement standard SHA-256 and random trace/span ID generation with parent/child helpers.
- [ ] Rerun tests to green and commit.

### Task 2: Opt-in OTLP/HTTP JSON exporter

**Files:** `src/telemetry.hpp`, `src/telemetry.cpp`, `tests/telemetry_test.cpp`

- [ ] Write failing exporter tests: default disabled emits nothing/no file; explicit file has `resourceSpans`, `scopeSpans`, `spans`, required IDs/timestamps/attributes/status, and no prompt fields.
- [ ] Run tests to red, then implement JSON escaping, typed resource/span attributes, and explicit `UOR_OTEL_OUT` file sink. No network code or default endpoint.
- [ ] Rerun tests to green and commit.

### Task 3: Instrumented CUDA consumer and evidence

**Files:** `src/instrumented.hpp`, `src/instrumented.cpp`, `examples/instrumented.cpp`, `tests/instrumented_test.cpp`, `docs/telemetry/uor-conventions.md`, `docs/labs/uor-telemetry/README.md`, `docs/labs/uor-telemetry/evidence.json`

- [ ] Add failing consumer tests: build/probe/full-scan spans use correct operations and stable artifact digests but different execution IDs, with valid parent IDs.
- [ ] Implement thin wrappers over existing CPU functions, preserving status/results. Document local namespace, identity kinds, default-off, and future generic artifact-digest upstream path.
- [ ] Generate evidence with and without `UOR_OTEL_OUT`, confirm no file by default and valid JSON when enabled. Keep `make evidence` green and commit.

### Task 4: Delivery and CUDA advertising checklist

**Files:** `docs/loop/tasks/uor-telemetry/kimi-1/ORDER.md`, `docs/loop/tasks/uor-telemetry/kimi-1/REPORT.md`, `projects/uor-cuda-inverted-file/ADVERTISING.md`

- [ ] Document actual evidence, every NOT_RUN, and no upstream-acceptance claim.
- [ ] List remaining CUDA advertising gates: clean AOT build, real GPU parity, tuned matched controls on two architectures, repeated measurements with separate kernel/end-to-end scopes, raw artifacts, independent replication, then cuda-samples issue/Developer Forums/consider Foundation Sandbox.
- [ ] Run full host/documentation/scope checks and fresh whole-branch review before protected delivery.

## Self-review

The four tasks cover identifiers, exporter, consumer evidence, and advertising without touching model behavior. Default-off, ID semantics, OTLP shape, artifact identity, and reproducibility each have direct tests. CUDA advertising remains a separate hardware-verified gate rather than an assumed result of adding telemetry.
