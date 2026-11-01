---
id: uor-community-evidence-releases
title: Publish the CUDA and OpenTelemetry evidence bundles as versioned releases
milestone: research
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: [uor-cuda-inverted-file, uor-otel-artifact-convention]
done_class: host
scope: [projects/uor-cuda-inverted-file/**, docs/telemetry/**, docs/loop/tasks/uor-community-evidence-releases/**, docs/roadmap/items/uor-community-evidence-releases.md, docs/roadmap/issues.json]
---

## Why it matters

The standalone inverted-file CUDA example and its opt-in OpenTelemetry experiment have reviewed correctness and official-collector interoperability evidence, but evidence in repository folders is not a downloadable release. Versioned tags, source/evidence archives, release notes, and checksums give outside reviewers a concrete artifact to inspect without confusing either bundle with the UOR-R4 model's M8 release.

## What is missing

Nothing for this item. Both actual public releases target protected `main` merge `4daf787f3064e348d5659c4f623429aa8b5c9f30`, are non-draft/non-prerelease and not Latest, and their published assets passed download, checksum, and extracted-test verification: [`cuda-inverted-file-v0.1.0`](https://github.com/UOR-Foundation/uor-r4/releases/tag/cuda-inverted-file-v0.1.0) with archive SHA-256 `dff4c9043385abbdc36d388593d35168ff6843227b2e8064ffcfd3450ce9307b`, and [`otel-artifact-v0.1.0`](https://github.com/UOR-Foundation/uor-r4/releases/tag/otel-artifact-v0.1.0) with archive SHA-256 `e7175c276bb6fc7c350d5ca242f53d1a4e5e7f0b1e74dd4b43caf073185d8fed`. SDK and multi-runtime interoperability and a real loaded-model example remain NOT_RUN; local `uor.*` attributes remain experimental. No NVIDIA or OpenTelemetry submission or acceptance is implied.

## Done when

- `cuda-inverted-file-v0.1.0` and `otel-artifact-v0.1.0` are non-draft GitHub releases tied to the protected `main` commit containing their release metadata, and neither is presented as the model's M8/latest release.
- Each release has a reviewed source/evidence archive, accurate notes and limitations, and a SHA-256 manifest; published assets are downloaded again and their checksums and archive contents verified.
- The CUDA archive passes its host and telemetry checks from the extracted archive. The OpenTelemetry archive passes its host telemetry checks and preserves default-off behavior; neither check is represented as a new GPU or collector run.
- Release documentation distinguishes the measured CUDA core from later metadata changes and keeps general-kNN, end-to-end-speedup, production-telemetry, NVIDIA-endorsement, and upstream-acceptance claims out of scope.
- Fresh exact-head reviews and protected PRs deliver the metadata and final release records. No upstream issue, pull request, forum post, or paid compute is part of this item.

## How to start

- Reuse the accepted CUDA and collector evidence; do not rerun GPU measurements or change implementation source for packaging.
- Use `git archive` on the reviewed merge commit, attach archives and SHA-256 manifests with `gh release create`, then download the assets into a separate directory and verify them.
- Keep both releases non-draft and use `--latest=false`; the historical model releases and future M8 model release are separate.

## Rules that apply

- **Preservation:** no model, artifact, CUDA/telemetry implementation, Claude-owned, generated-binary, or production-deployment changes.
- **Evidence:** no new performance, portability, interoperability, model-capability, endorsement, or upstream-acceptance claim follows from publishing a release.
- **Delivery:** tag only protected `main`, never an unreviewed branch; a checksum mismatch or unavailable asset is a failed verification, not a pass.

## Sources

- Issue #3475 and `projects/uor-cuda-inverted-file/OUTREACH.md`
- Issue #3486 and `docs/telemetry/model-artifact-convention.md`
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-3/REPORT.md`
- `docs/labs/uor-telemetry/collector-results.json`
