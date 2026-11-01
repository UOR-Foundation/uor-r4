---
id: uor-otel-artifact-convention
title: Verify OTLP collection and prepare a generic model-artifact digest convention
milestone: research
area: mechanism
status: done
open_to: lab
needs_gpu: false
depends_on: [uor-telemetry]
done_class: host
scope: [projects/uor-cuda-inverted-file/examples/otel/**, docs/telemetry/model-artifact-convention.md, docs/telemetry/uor-conventions.md, docs/labs/uor-telemetry/**, docs/loop/tasks/uor-otel-artifact-convention/kimi-1/**, docs/superpowers/plans/2026-11-01-otel-artifact-convention.md, docs/roadmap/items/uor-otel-artifact-convention.md, docs/roadmap/README.md, docs/roadmap/issues.json]
---

## Why it matters

The local telemetry consumer should work with the actual OpenTelemetry ecosystem, and any upstream proposal should solve a vendor-neutral problem: identifying which loaded model bytes produced a result. A small collector check and careful proposal are more useful than treating hexadecimal hashes as trace IDs.

## What is missing

**Collector ingestion and decoded-span verification pass** with official Collector Contrib 0.164.0: all four operations, parent links, and artifact metadata were decoded with no dropped attributes. A vendor-neutral proposal is drafted in `docs/telemetry/model-artifact-convention.md`. SDK/multi-runtime interoperability and a real loaded-model example remain **NOT_RUN**; no proposal has been submitted or accepted upstream. The local `uor.*` attributes remain experimental and distinct from execution IDs and future standard names.

## Done when

- The official OpenTelemetry Collector accepts an explicitly generated local trace over loopback HTTP and decodes all four spans, parent links, and artifact metadata.
- The default application remains telemetry-free; no production endpoint, SDK, or deployment is added.
- A vendor-neutral proposal identifies the loaded-artifact digest, algorithm, byte size, and identity scope without UOR geometry, hash-derived execution IDs, prompt/completion text, or user-turn hashes.
- Current upstream contribution requirements, privacy/cardinality limits, interoperability evidence, and remaining SDK/multi-runtime gaps are recorded accurately.
- Fresh exact-head review approves protected delivery. No upstream issue or pull request is opened automatically.

## How to start

- Use `projects/uor-cuda-inverted-file/examples/otel/README.md` with the pinned official collector image.
- Read the current `semantic-conventions-genai` contribution guide and existing registry before drafting names.
- Preserve the actual collector log and every NOT_RUN, including production export and multi-runtime/SDK interoperability.

## Rules that apply

- **Opt-in:** no default telemetry, endpoint, collector, production deployment, or user content.
- **Identity:** execution IDs remain random; content digests remain resource metadata.
- **Upstream:** a generic convention is a proposal, never an acceptance claim. UOR-specific geometry stays local.

## Sources

- `docs/telemetry/uor-conventions.md`
- `docs/labs/uor-telemetry/`
- OpenTelemetry Collector Contrib `0.164.0`
- [GenAI semantic-conventions contributing guide](https://github.com/open-telemetry/semantic-conventions-genai/blob/main/CONTRIBUTING.md)
