# ORDER: register community evidence releases, kimi-4

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Base: `83d54d168` from fresh `origin/main`. Owner request: make the CUDA and OpenTelemetry bundles actual releases.

## Goal

Register the separate roadmap item for publishing both evidence bundles. This is the item/scope PR required so the later release-metadata PR has its item at the merge base. It changes no public release status, implementation, evidence, or upstream submission state.

## Scope

- New `docs/roadmap/items/uor-community-evidence-releases.md`
- Canonical `docs/roadmap/issues.json` and `docs/roadmap/README.md`
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-4/**`

No release notes, package implementation, model, artifact, API, worker, engine, desktop, `.github/**`, Claude-owned, production-deployment, NVIDIA-submission, or OpenTelemetry-submission changes.

## Cancel probe

Stop if the release item or either proposed tag already exists on protected `main`, or if its dependencies are absent. Issue #3492 already exists from canonical roadmap sync and must be mapped, not duplicated. The probe found the item absent from `main`, both dependencies present, and no conflicting tags.

## Done when

- The new item is `ready`, depends on `uor-cuda-inverted-file` and `uor-otel-artifact-convention`, and accurately names protected publication and read-back verification as its remaining work.
- `docs/roadmap/issues.json` maps it to existing issue #3492 and the generated roadmap renders with 125 items.
- Roadmap checks, claim wording, repository gates, local policy, and fresh exact-head review pass.
- The protected PR merges with `References #3475` and `References #3492`; neither issue closes here and no release is claimed.

## Stop and report

Stop after registration merges. Do not add release metadata to this PR, mark #3492 done, create tags/releases, or submit upstream. If any file outside this registration scope is needed, stop and re-scope first.
