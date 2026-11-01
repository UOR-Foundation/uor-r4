# REPORT: OTLP interoperability and generic artifact convention, kimi-1

Lab: kimi. Item: #3486. Order: [ORDER.md](ORDER.md).

## Negatives and limits first

- Docker Desktop was absent and its credential helper was misconfigured for this CLI. A fresh task-local config plus the existing Colima runtime was used; no credentials were read or copied.
- The macOS-to-VM port path failed. Direct container IP was inappropriate because the collector binds only loopback. The successful check ran the curl client in the collector's own network namespace, reaching `127.0.0.1:4318` without weakening the bind address or opening an external endpoint.
- The first curl attempt targeted a read-only output mount. The final command streams output locally and is the command retained in the example README.
- This validates collector transport only. OpenTelemetry SDK use, multiple runtimes, a real loaded language model, production export, and upstream acceptance remain **NOT_RUN/NOT_CLAIMED**. The fixed codebook is not a model artifact.

## Measured result

Official `otel/opentelemetry-collector-contrib:0.164.0` at image digest `sha256:340c9ca6a36617084c6661bdc4f0fe92b7cda5a805d61673ec8db375fddc522b` accepted the explicitly generated trace with **HTTP 200** and `{"partialSuccess":{}}`.

The debug log decoded all four operations:

- `uor.cuda.index.build` as the root span;
- `uor.cuda.index.update`, `uor.cuda.index.probe`, and `uor.cuda.index.full_scan` with the build span as parent;
- shared trace ID `2bcf737d44ec9b619db3c1db79488566` and build span ID `5099af27a3b4ca7b`;
- codebook SHA-256 `9d98bd50f46cdedd5c481a8d696a609936bbc540997e7eaef6af2bdb3bc15597` and byte size `512`;
- zero dropped attributes per span.

The collector was stopped and removed. The application remains default-off and contains no network exporter. All three host telemetry test executables pass.

Machine-readable evidence is in `docs/labs/uor-telemetry/collector-results.json`; original logs/config/trace are in `local/kimi/otel-release-1/` with SHA-256 verification and in the [project store](https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/4df3c78a8d4652ec4b713dc621e5ddf67b7d8cd9), verified again after download.

## Proposal and next decision

`docs/telemetry/model-artifact-convention.md` proposes resource-level loaded-artifact digest, algorithm, byte size, and identity-kind attributes. It excludes UOR geometry, hash-derived execution IDs, prompt/completion text, user-turn hashes, and high-cardinality metric labels. The current GenAI registry and contribution guide were read directly.

The next responsible step is a real loaded-model consumer and a second independent runtime/example, followed by a discussion issue—not an automatic upstream PR. No issue, Slack post, production deployment, or upstream-acceptance claim was made here.
