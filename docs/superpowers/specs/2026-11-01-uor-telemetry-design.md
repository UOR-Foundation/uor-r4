# Opt-in UOR telemetry design

## Intent

Implement the approved OpenTelemetry recommendation as a real consumer in the standalone inverted-file example. Provide local OTLP/HTTP JSON evidence with random execution IDs and artifact hashes as attributes, without enabling telemetry by default or touching model/API/worker/engine/Claude code.

## Required behavior

- Default disabled: no environment setting means no exporter, no output file, and no network endpoint.
- Explicit local file sink only: `UOR_OTEL_OUT` selects the output file. There is no default endpoint, socket, collector, or phone-home path. An explicit user-run curl may post that file to a local collector later.
- Random 16-byte trace IDs and 8-byte span IDs, lowercase hex, never derived from content hashes. Parent IDs form a real parent/child relationship.
- Full artifact digests as attributes with algorithm and identity kind. No prompt/completion text or user-turn hashes.
- Valid OTLP/HTTP JSON Protobuf shape: `resourceSpans`, `scopeSpans`, `spans`, `traceId`, `spanId`, `parentSpanId`, timestamps, attributes, and status.
- Four operations for the consumer: `uor.cuda.index.build`, `uor.cuda.index.update`, `uor.cuda.index.probe`, and `uor.cuda.index.full_scan`. Keep the four CUDA kernel names unchanged.

## Constraints

- No model, artifact, API, worker, engine, Claude-owned, `.github/**`, desktop, or production deployment changes.
- No third-party dependency, OpenTelemetry SDK, gRPC stack, Python, committed binary, or secret.
- No telemetry in the desktop binary; the standalone example is the only consumer in this unit.
- No performance, portability, energy, cross-library, or upstream-acceptance claim.
- GPU timing remains NOT_RUN on this machine.

## Implementation shape

`src/sha256.*` implements standard SHA-256 for artifact bytes and known-vector tests. `src/telemetry.*` implements ID generation, JSON escaping, resource/span attributes, and an explicit file exporter. `src/instrumented.*` wraps the existing CPU index functions without changing their behavior. `examples/instrumented.cpp` shows opt-in usage, and `tests/telemetry_test.cpp` verifies IDs, defaults, schema, attributes, and reproducible evidence.

`docs/telemetry/uor-conventions.md` defines the local UOR namespace, identity distinctions, allowed attributes, default-off requirement, and the future upstream path for a generic loaded-model artifact digest convention.

## Verification

- SHA-256 empty and `abc` standard vectors; artifact digest stability across identical inputs.
- ID widths, nonzero values, root uniqueness, and no digest reuse as IDs.
- Default disabled creates no file and no endpoint.
- Explicit file output has valid OTLP resource/scope/span shape, expected attributes, and correct parent IDs.
- Two runs over the same artifact keep artifact digests identical but execution IDs different.
- Existing host/evidence tests stay green; clean-copy verification remains independent of telemetry.
