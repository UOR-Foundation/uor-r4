# ORDER: OTLP interoperability and generic artifact convention, kimi-1

Lab: kimi. Item: `uor-otel-artifact-convention` (#3486). Base: current `origin/main`. Owner request: get the CUDA package and OpenTelemetry integration ready for public presentation.

## Goal

Verify that the official OpenTelemetry Collector decodes the standalone example's OTLP/HTTP JSON, then prepare a vendor-neutral loaded-model artifact-digest proposal without making an upstream acceptance or production-telemetry claim.

## Scope

- `projects/uor-cuda-inverted-file/examples/otel/**`
- `docs/telemetry/model-artifact-convention.md`
- `docs/telemetry/uor-conventions.md`
- `docs/labs/uor-telemetry/**`
- `docs/loop/tasks/uor-otel-artifact-convention/kimi-1/**`
- `docs/superpowers/plans/2026-11-01-otel-artifact-convention.md`
- `docs/roadmap/items/uor-otel-artifact-convention.md`
- Canonical `docs/roadmap/README.md` and `docs/roadmap/issues.json`
- Task-local collector evidence under ignored `local/kimi/otel-release-1/`

No model, artifact, API, worker, engine, desktop, `.github/**`, Claude-owned, production-deployment, or telemetry-consumer behavior changes. No upstream issue/PR, Slack post, or public announcement in this unit.

## Cancel probe

Telemetry remains off unless `UOR_OTEL_OUT` is explicitly supplied. The collector binds only loopback and runs only for the explicit check. The POST alone is insufficient: the official collector log must decode all four spans, correct parent links, and the 512-byte codebook digest. Stop the container after the check.

## Done when

- Official Collector Contrib 0.164.0 accepts and decodes the trace; the log and exact image/config/source identities are preserved.
- The local JSON and original host tests remain green, with no new application dependency or default endpoint.
- The generic proposal and record distinguish execution/artifact identity and document privacy, cardinality, upstream process, and every remaining NOT_RUN.
- Fresh exact-head review and protected delivery verify the evidence on fresh main.

## Stop and report

Stop after interoperability and proposal readiness. Do not open an upstream issue/PR, claim acceptance, run production telemetry, or advertise CUDA performance in this unit.
