# Opt-in standalone CUDA telemetry

Lab: kimi. Item: [#3478](https://github.com/UOR-Foundation/uor-r4/issues/3478). Base: `ebe0930ba`. Date: 2026-11-01.

## What this demonstrates

The standalone inverted-file example now has an opt-in, local-only OTLP/HTTP JSON consumer. It preserves existing CPU behavior, uses random execution IDs, and records the fixed codebook's SHA-256 as resource metadata. Build is the root span; update, probe, and full-scan are its children in the same trace.

No model, artifact, API, worker, engine, desktop, deployment, or Claude-owned code changed. This is not a generic OpenTelemetry SDK, a collector integration, a GPU instrument, or an upstream acceptance.

## Measured checks

- `make -C projects/uor-cuda-inverted-file telemetry-test`: all three test executables passed. They cover standard SHA-256 vectors, random ID width/uniqueness, default-off export, JSON escaping and refusal, all four operation spans, parent links, CPU update/scan parity, and stable artifact identity across different traces.
- `make -C projects/uor-cuda-inverted-file evidence`: PASS. Two host runs and one clean-copy run produced identical output with SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`.
- Running the example in an empty directory with `UOR_OTEL_OUT` unset left the directory empty and printed `telemetry disabled: no file or endpoint created`.
- Two explicit `UOR_OTEL_OUT=trace.json` runs each produced one `jq`-validated JSON document containing four spans. Every child pointed to the emitted build span; IDs had the required widths and lowercase hex form.
- The final example traces were `bb28418e60db4274f75b625175205b80` and `52847b5c4cfd474c106970a6e411e2d6`. Both recorded codebook byte size 512 and the same SHA-256, `9d98bd50f46cdedd5c481a8d696a609936bbc540997e7eaef6af2bdb3bc15597`.

See [evidence.json](evidence.json) for the machine-readable result and [the local conventions](../../telemetry/uor-conventions.md) for the identity and privacy boundaries.

## NOT_RUN and limits

- CUDA AOT compilation: **NOT_RUN** (`nvcc` unavailable on Darwin arm64).
- CUDA Driver API execution, CPU/GPU parity, and GPU timing: **NOT_RUN**.
- Collector ingestion and interoperability with an OpenTelemetry SDK: **NOT_RUN**.
- Network transmission: no network code or endpoint was added; no collector was run.
- Production telemetry, performance, energy, portability, model capability, and upstream acceptance: not measured or claimed.

The fixed-codebook digest identifies only those 512 payload bytes, not a model, executable, source revision, or run. Random execution IDs are correlation identifiers, not UOR addresses or security tokens. Each export creates a collision-resistant, exclusively owned temporary directory and renames only its own complete file over the current trace where the platform supports replacement; concurrent writers cannot share or clobber a fixed temporary path, and a failed replacement preserves the prior destination. This is intended for example use, not production export. `TelemetryFailure` reports an export I/O failure rather than mislabeling it as CUDA; CPU operation errors take precedence, and completed CPU side effects are not rolled back. Timestamps use wall-clock start plus steady-clock elapsed time; disabled wrappers call only the original CPU operation.

## Next upstream step

Evaluate a vendor-neutral loaded-model artifact-digest attribute with the OpenTelemetry semantic-conventions community, after checking prior art, privacy/cardinality, and real SDK/collector interoperability. Do not submit UOR hashes as trace or span IDs.
