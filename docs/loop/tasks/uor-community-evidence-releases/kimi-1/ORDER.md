# ORDER: prepare versioned evidence releases, kimi-1

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Base: `3f64c679cfaca4411779d819019747fd44a7a854` from fresh `origin/main`, including registration #3494. Owner request: make the CUDA and OpenTelemetry bundles actual releases.

## Goal

Prepare and verify the two `0.1.0` release definitions and source/evidence archives, deliver the metadata through a protected PR, then publish and read back both releases. This order covers the metadata PR; the publication record and item close follow in the next order.

## Scope

- `projects/uor-cuda-inverted-file/README.md`, `CITATION.cff`, `OUTREACH.md`, and new `RELEASE.md`
- `docs/telemetry/uor-conventions.md` and new `RELEASE.md`
- `docs/roadmap/items/uor-community-evidence-releases.md` and canonical roadmap sync/render outputs
- `docs/loop/tasks/uor-community-evidence-releases/kimi-1/**`
- Ignored scratch archives and extracted copies under `local/kimi/community-evidence-releases/`

No model, artifact, CUDA/telemetry implementation, API, worker, engine, desktop, `.github/**`, Claude-owned, generated-binary, production-deployment, NVIDIA-submission, or OpenTelemetry-submission changes.

## Cancel probe

Stop if either tag already exists, if CUDA source rather than documentation differs from measured implementation `719a9a7861deb595dfc5269ffec8a5e7e53cb39b`, or if the accepted evidence is absent. The completed probe found no conflicting tags, only post-measurement CUDA documentation changes, and passing host/telemetry/evidence checks.

## Done when

- Release notes and citation metadata identify `0.1.0`, exact archive contents, measured scopes, limitations, and the lack of upstream submission/acceptance.
- Trial `git archive` bundles contain no generated binaries or scratch files; extracted CUDA host/evidence and OpenTelemetry telemetry/default-off checks pass.
- Claim wording, roadmap, and relevant repository checks pass at the exact head; a fresh adversarial review approves it.
- A non-draft protected PR merges the metadata. Actual releases and their read-back verification are explicitly pending at this seam.

## Stop and report

Stop after the metadata merge. Do not mark the roadmap item done or claim a release exists before `gh release create` succeeds and published assets pass download/checksum verification. Any required implementation change, upstream submission, or GPU rerun is outside this order.
