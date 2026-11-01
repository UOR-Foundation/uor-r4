# REPORT: opt-in standalone CUDA telemetry, kimi-1

Lab: kimi. Order: [ORDER.md](ORDER.md). Item: [#3478](https://github.com/UOR-Foundation/uor-r4/issues/3478). Base: `ebe0930ba`.

This branch adds an opt-in local OTLP/HTTP JSON consumer to the standalone inverted-file example and documents the remaining CUDA release gates. No model, artifact, API, worker, engine, desktop, `.github/**`, production deployment, or Claude-owned file changed. No GPU/model compute, paid resource, collector, upstream issue, or public announcement was used.

## Negatives and limits first

- The initial single-span exporter overwrote earlier operations. The consumer now serializes the accumulated spans into one valid OTLP JSON document, with the build span as parent of update, probe, and full-scan. New tests caught the missing batch API before it was added.
- Initial consumer compilation exposed missing namespace imports; these were corrected without changing the index implementation.
- The resource initially named the CUDA driver even though the wrappers execute the CPU oracle. It now says `cpu-oracle`; no GPU execution or timing is claimed.
- `nvcc` is unavailable on Darwin arm64. Both `make cuda` and `make cuda-test` stop with `NOT_RUN: nvcc is unavailable; CUDA compilation and tests were not performed`. AOT compilation, Driver API execution, CPU/GPU parity, and GPU timing are **NOT_RUN**.
- Collector ingestion, OpenTelemetry SDK interoperability, production export, performance, energy, portability, and upstream acceptance are **NOT_RUN** or **NOT_CLAIMED**. The JSON file is not evidence that a collector accepted it.
- The first direct `mdbook build` lacked the required staged source. The documented `book-stage` pipeline was then run successfully. mdBook also reported the existing preprocessor/tool version mismatch (`0.4.36` versus `0.4.40`) without failing the build.
- A completion edit temporarily replaced the mandatory roadmap `What is missing` heading; the required heading was restored and roadmap validation passed. Normal roadmap sync marked #3478 done and closed it after the host acceptance criteria were met; protected branch delivery is reported separately rather than inferred from that issue state.

## What changed

- Added standard SHA-256, random 16-byte trace IDs and 8-byte span IDs, lowercase hexadecimal encoding, and parent/child helpers. Digests remain metadata and never become execution IDs.
- Added a standard-library-only OTLP/HTTP JSON file exporter with escaping, typed attributes, validation, and batch output. No endpoint, socket, collector, SDK, network code, third-party dependency, prompt/completion text, or user-turn hash was added.
- Added thin instrumented wrappers for build, update, probe, and full-scan. Update matches a fresh build; probe/full-scan results retain parity. Telemetry remains disabled unless an output path is explicitly supplied; the example uses `UOR_OTEL_OUT`.
- Documented local UOR attributes, execution/artifact identity distinctions, the privacy boundary, and a future generic loaded-model artifact-digest proposal. No UOR geometry or hash-shaped execution ID is proposed upstream.
- Added `projects/uor-cuda-inverted-file/ADVERTISING.md`: do not advertise until real AOT/GPU parity, tuned matched controls, two architectures and driver versions, scoped raw measurements, independent replication, and external-consumer validation. Only then consider a cuda-samples clarification issue, Developer Forums thread, and Foundation Sandbox application.

## Measurements and checks

- `make -C projects/uor-cuda-inverted-file telemetry-test`: **PASS**, all three executables.
- Default example run in an empty directory: **PASS**, directory remained empty; no telemetry file or endpoint was created.
- Two explicit file runs: **PASS**, each one `jq`-validated JSON document with four spans and correct emitted parent links. Trace IDs differed (`7043687a3b0e22ee80c8ca32c470b7d1`, `652e308aa063d3812f894cbd0cf723e3`); both retained codebook SHA-256 `9d98bd50f46cdedd5c481a8d696a609936bbc540997e7eaef6af2bdb3bc15597`.
- `make -C projects/uor-cuda-inverted-file evidence`: **PASS**. Two host runs and one clean-copy run produced identical output, SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d` for each log.
- `cargo test --locked -p xtask`: **220 passed, 0 failed**.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: **PASS**.
- `cargo run -q --locked -p xtask -- roadmap check`: **123 items OK**; `roadmap render --check` current; `docs-catalog --check` reports 1,175 records, 20 lines, and zero missing paths.
- `book-stage`, `mdbook-mermaid install .`, `mdbook build`, and `book-check`: **104 pages and 70 figures staged; 105 pages, 0 broken links**.
- `git diff --check`: **PASS**.

Machine-readable host evidence is in `docs/labs/uor-telemetry/evidence.json`. Test success is not model capability, GPU verification, production telemetry qualification, or upstream acceptance.

## Delivery and next decision

Fresh exact-head review, required PR checks, protected merge, and fresh-main verification are part of this unit and are not presumed by this report. The next OpenTelemetry decision is whether to pursue a vendor-neutral artifact-digest convention after prior-art/privacy review and real SDK/collector interoperability. The next CUDA decision is a separately registered hardware correctness run, not advertising.
