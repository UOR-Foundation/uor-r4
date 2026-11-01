---
id: uor-telemetry
title: Add opt-in artifact telemetry to the standalone CUDA example
milestone: research
area: mechanism
status: done
open_to: lab
needs_gpu: false
depends_on: [uor-cuda-inverted-file]
done_class: host
scope: [projects/uor-cuda-inverted-file/**, docs/telemetry/uor-conventions.md, docs/labs/uor-telemetry/**, docs/loop/tasks/uor-telemetry/kimi-1/**, docs/superpowers/specs/2026-11-01-uor-telemetry-design.md, docs/superpowers/plans/2026-11-01-uor-telemetry.md, docs/roadmap/items/uor-telemetry.md]
---

## Why it matters

A local, opt-in OpenTelemetry consumer can demonstrate how artifact identity relates to execution identity without changing the model or sending data anywhere. It gives the standalone CUDA example a small, dependency-free integration that others can inspect before considering a generic upstream semantic convention.

## What is missing

**The host-only consumer is complete.** Explicit `UOR_OTEL_OUT` output contains valid OTLP/HTTP JSON with all four operations, real parent links, random execution IDs, and stable codebook SHA-256 metadata. Default operation creates no file or endpoint, and the existing host/clean-copy evidence passes. GPU execution and collector interoperability remain **NOT_RUN** and are outside this unit. The local namespace and the separate generic upstream path are documented in `docs/telemetry/uor-conventions.md`; the CUDA advertising gates are in `projects/uor-cuda-inverted-file/ADVERTISING.md`.

## Done when

- Default operation creates no telemetry file, endpoint, socket, or collector dependency.
- Explicit `UOR_OTEL_OUT` output contains valid OTLP/HTTP JSON with build/update/probe/full-scan operations, correct parent links, and random execution IDs.
- Repeated runs preserve the codebook's SHA-256 identity but use different execution IDs; no prompt, completion, or user-turn hash is collected.
- Existing host parity and clean-copy evidence pass, while CUDA compilation and execution remain explicitly NOT_RUN without `nvcc` and a GPU.
- The local attribute namespace and the separate, generic loaded-model artifact-digest upstream proposal are documented without an acceptance claim.

## How to start

- Use the approved `docs/superpowers/specs/2026-11-01-uor-telemetry-design.md` and its implementation plan.
- Keep model, artifact, API, worker, engine, desktop, Claude-owned, and `.github/**` paths unchanged.
- Run `make -C projects/uor-cuda-inverted-file telemetry-test evidence` and inspect the actual JSON before delivery.

## Rules that apply

- **Opt-in:** no default telemetry, endpoint, network code, third-party SDK, secret, or production deployment.
- **Identity:** trace/span IDs are random execution identifiers; SHA-256 and other content digests remain metadata.
- **Scope:** this is a standalone host consumer, not model instrumentation, a performance result, or an upstream acceptance.

## Sources

- `docs/superpowers/specs/2026-11-01-uor-telemetry-design.md`
- `docs/telemetry/uor-conventions.md`
- `projects/uor-cuda-inverted-file/README.md`
- OpenTelemetry [trace IDs](https://opentelemetry.io/docs/specs/otel/trace/api/#spancontext) and [OTLP/HTTP JSON](https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding) specifications
