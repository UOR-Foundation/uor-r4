# Opt-in UOR Artifact Telemetry 0.1.0

A downloadable source-and-evidence release of the standalone OTLP/HTTP JSON example, its explicit loopback OpenTelemetry Collector check, and a vendor-neutral **draft** loaded-artifact identity convention. This is not an OpenTelemetry SDK, production exporter, accepted semantic convention, or the UOR-R4 model's M8 release.

## Assets and verification

- `uor-otel-artifact-0.1.0.tar.gz`: the standalone consumer and its required source, the collector configuration, local conventions and draft proposal, collector results, and task records. No generated binaries, credentials, model checkpoint, corpus, or user content are included.
- `SHA256SUMS`: the SHA-256 manifest for the archive.

Download both assets, then verify and extract them:

```sh
shasum -a 256 -c SHA256SUMS
tar -xzf uor-otel-artifact-0.1.0.tar.gz
cd uor-otel-artifact-0.1.0/projects/uor-cuda-inverted-file
make telemetry-test
make instrumented-example
```

Telemetry is disabled unless `UOR_OTEL_OUT` is explicitly set. The default command above tests the application without creating a telemetry file; the telemetry test suite also checks default-off behavior, random execution IDs, parent links, JSON refusal paths, and CPU parity. To create the local OTLP/HTTP JSON file explicitly:

```sh
UOR_OTEL_OUT=build/trace.json make instrumented-example
```

To repeat the collector experiment, follow `examples/otel/README.md`. It uses the official Collector Contrib image pinned by digest, binds only loopback, requires Docker, and must be stopped after inspection. Repeating it is optional; the release's recorded collector result is not represented as a new run.

Release tag: `otel-artifact-v0.1.0`, targeting the protected `main` merge commit that contains this release definition.

## Recorded collector evidence

Official `otel/opentelemetry-collector-contrib:0.164.0`, image digest `sha256:340c9ca6a36617084c6661bdc4f0fe92b7cda5a805d61673ec8db375fddc522b`, accepted the explicitly generated local trace with HTTP 200 and decoded all four spans. The build span is the parent of update, probe, and full-scan; every span has zero dropped attributes.

The resource identifies the fixed codebook as 512 bytes with SHA-256 `9d98bd50f46cdedd5c481a8d696a609936bbc540997e7eaef6af2bdb3bc15597`. That digest is artifact metadata, never a trace or span ID. The example instruments the CPU oracle, not CUDA kernels, and contains no application network exporter.

Evidence archive: https://huggingface.co/datasets/caseyallard/uor-r4-store/commit/4df3c78a8d4652ec4b713dc621e5ddf67b7d8cd9

## Draft convention and remaining work

`docs/telemetry/model-artifact-convention.md` proposes optional loaded-artifact digest, algorithm, byte-size, and identity-kind resource attributes for external discussion. The local `uor.*` names remain experimental and separate from proposed `gen_ai.*` names.

Before any upstream YAML pull request, the project still needs a real loaded-model consumer and a second independent runtime/example, plus privacy and cardinality review. A discussion issue should come first. No OpenTelemetry issue, Slack post, or pull request is submitted by this release, and no upstream acceptance is claimed.

Production export, SDK interoperability, multi-runtime interoperability, GPU telemetry, prompt/completion collection, and model-capability evidence are outside this release's measured scope.
