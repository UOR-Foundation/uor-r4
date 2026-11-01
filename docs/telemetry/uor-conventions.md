# Local UOR telemetry conventions

Status: experimental, repository-local attributes for the standalone inverted-file example. These are **not** OpenTelemetry semantic conventions, an upstream proposal, or evidence of upstream acceptance.

## Consumer and privacy boundary

The only consumer is `projects/uor-cuda-inverted-file`. Telemetry is disabled unless the caller explicitly supplies an output path. The example reads `UOR_OTEL_OUT`; an unset variable means no output file, endpoint, socket, collector, or network code. The file sink writes OTLP/HTTP JSON locally and replaces it with the current in-memory trace after each operation. It is not a production exporter, queue, uploader, or OpenTelemetry SDK.

The consumer emits operation names, numeric operation parameters, status, timing, and fixed-codebook metadata. It does **not** emit prompt or completion text, user-turn hashes, credentials, or environment contents. This example instruments the **CPU oracle**, not GPU execution. Collector ingestion was verified separately with official Collector Contrib 0.164.0 over loopback; GPU timing remains **NOT_RUN**.

## Execution identity is not artifact identity

| Concept | Representation | Rule |
|---|---|---|
| Trace ID | Random 16 bytes, encoded as 32 lowercase hex characters | New for each instrumented session; never derived from content. |
| Span ID | Random 8 bytes, encoded as 16 lowercase hex characters | New for each operation; never derived from content. |
| Parent span ID | The build/root span's random ID | Links operation children in the same trace. |
| Artifact payload identity | `uor.artifact.payload_sha256` | SHA-256 of the exact fixed-codebook payload bytes; remains metadata. |
| Artifact identity kind | `uor.artifact.identity_kind` | Here, `fixed-codebook`; this is not a whole-model or executable identity. |

OpenTelemetry trace and span IDs identify execution. SHA-256, BLAKE3, source revisions, model versions, and UOR addresses answer different identity questions and must not be truncated or renamed into execution IDs. A digest of 64 hex characters is not a valid trace ID or span ID. Random IDs are correlation identifiers, not security tokens.

## Local attributes

Resource attributes:

- `uor.schema.version`: literal `uor.telemetry/1` for this local experiment.
- `uor.artifact.payload_sha256`: full lowercase SHA-256 digest of the codebook bytes.
- `uor.artifact.digest.algorithm`: literal `sha256`.
- `uor.artifact.identity_kind`: literal `fixed-codebook`.
- `uor.artifact.byte_size`: decimal byte count, `512` for this codebook (16 blocks × eight 32-bit integers).
- `uor.serving.kernel`: literal `cpu-oracle`; it does not claim a CUDA kernel ran.

Span names are `uor.cuda.index.build`, `uor.cuda.index.update`, `uor.cuda.index.probe`, and `uor.cuda.index.full_scan`. These names describe the index API family; the current consumer calls its CPU implementations. `uor.operation.status` is the typed operation result. Update spans include `uor.operation.moved_token`; scan spans include `uor.operation.k`.

## A useful upstream path

Do not propose UOR-specific geometry, addresses, or hash-shaped execution IDs to OpenTelemetry. A potentially reusable contribution is a **generic loaded-model artifact-digest convention**: an optional resource attribute identifying the exact bytes loaded by an inference process, with explicit digest algorithm and identity scope. It would leave execution IDs random and would not prescribe UOR, SHA-256, a model format, or collection of user content.

The official Collector Contrib 0.164.0 accepted and decoded this example's four spans over loopback with no dropped attributes, establishing collector—not SDK or multi-runtime—interoperability. Before an upstream proposal, check current model/resource attributes and prior art, produce a real loaded-model example plus a second runtime, and obtain privacy/cardinality review. See [the generic draft](model-artifact-convention.md). No upstream issue or pull request has been opened by this unit.

The checked example, collector evidence, and draft are packaged in [`otel-artifact-v0.1.0`](https://github.com/UOR-Foundation/uor-r4/releases/tag/otel-artifact-v0.1.0), with [download and verification instructions](RELEASE.md). Publishing that archive does not change the draft's status or imply upstream acceptance.
