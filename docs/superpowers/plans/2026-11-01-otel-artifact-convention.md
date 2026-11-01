# OpenTelemetry Artifact Convention Readiness Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task with TDD and a final whole-branch review.

**Goal:** Verify real collector interoperability and prepare a vendor-neutral loaded-model artifact-digest proposal suitable for external review.

**Architecture:** Keep the existing dependency-free, default-off local JSON consumer unchanged. Add an explicit loopback-only collector experiment, inspect decoded spans, and document a generic proposal separately from local UOR attributes.

**Tech Stack:** C++17 example, OTLP/HTTP JSON, Docker/Colima, official OpenTelemetry Collector Contrib 0.164.0, Markdown/JSON evidence.

**Spec:** `docs/telemetry/uor-conventions.md` and `docs/roadmap/items/uor-otel-artifact-convention.md`.

## Global Constraints

- No model, artifact, API, worker, engine, desktop, `.github/**`, Claude-owned, production-deployment, or consumer-behavior changes.
- No default endpoint, application network exporter, prompt/completion text, user-turn hash, secret, or upstream-acceptance claim.
- Execution IDs remain random; content hashes never become execution IDs.
- Collector evidence must show decoded spans, not only an HTTP success response.

## Review Focus

- Collector configuration/version drift: pin and record the official image and loopback config (Task 1).
- HTTP success with invalid or incomplete data: inspect all four decoded operations, parents, and metadata (Task 1).
- UOR-specific branding in a generic proposal: check names, identity kinds, and rationale (Task 2).
- Privacy/cardinality and unsupported production claims: record exclusions and NOT_RUN scopes (Task 2).
- Stale upstream process: use the current GenAI repository's own guide and registry (Task 2).

---

### Task 1: Real collector interoperability

**Files:** `projects/uor-cuda-inverted-file/examples/otel/*`, `docs/labs/uor-telemetry/collector-results.json`, `docs/loop/tasks/uor-otel-artifact-convention/kimi-1/*`.

- [ ] Run the existing telemetry/host tests and generate an explicit local trace.
- [ ] Start official Collector Contrib 0.164.0 with loopback-only OTLP/HTTP; POST the trace and inspect the decoded four spans and metadata.
- [ ] Stop the container; preserve logs/config/identities and every NOT_RUN, then commit.

### Task 2: Generic proposal and delivery

**Files:** `docs/telemetry/model-artifact-convention.md`, collector evidence, ORDER/REPORT.

- [ ] Check the current upstream registry and contribution process, including reference scenarios and policy generation.
- [ ] Draft a vendor-neutral artifact identity convention with algorithm, digest, byte size, and identity scope, excluding user content and hash-derived execution IDs.
- [ ] Run complete host/documentation checks, obtain fresh exact-head review, deliver through protected PR, and verify on fresh main. Do not submit upstream automatically.
