# ORDER: opt-in standalone CUDA telemetry, kimi-1

Lab: kimi. Item: uor-telemetry (#3478). Base: `ebe0930ba`. Owner request: pursue the recommended OpenTelemetry integration and document the remaining CUDA advertising gates.

## Goal

Add a local-only, opt-in OTLP/HTTP JSON consumer to the standalone inverted-file example. Separate artifact identity from execution identity, preserve existing CPU behavior, and do not touch Claude's model work.

## Scope

- `projects/uor-cuda-inverted-file/**`
- `docs/telemetry/uor-conventions.md`
- `docs/labs/uor-telemetry/**`
- `docs/loop/tasks/uor-telemetry/kimi-1/**`
- `docs/superpowers/specs/2026-11-01-uor-telemetry-design.md`
- `docs/superpowers/plans/2026-11-01-uor-telemetry.md`
- `docs/roadmap/items/uor-telemetry.md`
- `docs/roadmap/README.md` and `docs/roadmap/issues.json` (canonical roadmap sync outputs)

No other paths. No model, artifact, API, worker, engine, desktop, `.github/**`, production deployment, third-party SDK, secret, or paid compute changes.

## Cancel probe

Telemetry must remain disabled unless `UOR_OTEL_OUT` explicitly selects a file. Trace IDs must be random 32-hex values and span IDs random 16-hex values; content hashes never become execution IDs. The existing four CUDA entry names and CPU results must not change. If `main` advances, merge forward and recheck only affected behavior.

## Done when

- Telemetry IDs, hashing, exporter, and consumer tests pass, including default-off, batched valid OTLP JSON, actual parent links, stable artifact identity, and different execution IDs.
- `make evidence` and clean-copy host verification remain green; `make cuda` and `make cuda-test` are explicitly NOT_RUN without `nvcc`.
- Documentation states the local namespace, privacy boundary, lack of collector verification, generic upstream path, and CUDA release/advertising gates.
- Fresh exact-head review, required checks, protected merge, and fresh-main verification are complete.

## Stop and report

Stop after this consumer and its release documentation are delivered. Do not enable production telemetry, run a collector, submit an upstream proposal, advertise CUDA performance, or start GPU/model work in this unit.
