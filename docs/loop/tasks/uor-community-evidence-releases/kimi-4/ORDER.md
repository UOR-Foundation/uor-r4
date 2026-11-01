# ORDER: publish and verify both evidence releases, kimi-4

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Base: protected `main` merge `4daf787f3064e348d5659c4f623429aa8b5c9f30` from approved #3495. Owner request: make both bundles actual releases.

## Goal

Create `cuda-inverted-file-v0.1.0` and `otel-artifact-v0.1.0` as non-draft GitHub releases at the protected metadata merge, attach final source/evidence archives and SHA-256 manifests, download and verify them, then deliver the public URLs/digests and close #3492 through a protected PR.

## Scope

- Remote GitHub tags/releases/assets only at `4daf787f3064e348d5659c4f623429aa8b5c9f30`, with `--latest=false`
- `projects/uor-cuda-inverted-file/{README.md,CITATION.cff,OUTREACH.md,RELEASE.md}`
- `docs/telemetry/{uor-conventions.md,RELEASE.md}`
- `docs/roadmap/items/uor-community-evidence-releases.md` and canonical sync/render outputs
- `docs/loop/tasks/uor-community-evidence-releases/kimi-4/**`
- Ignored final archives, notes, downloads, and extracted copies under `local/kimi/community-evidence-releases/final-4daf787f/`

No implementation, measured evidence, model, artifact, Claude-owned, owner-path, production deployment, NVIDIA submission, or OpenTelemetry submission changes.

## Cancel probe

Stop if `origin/main` is not the approved metadata merge, either tag already exists, or final source differs from the reviewed pending release files. A release creation is not success until both assets download and pass checksum/content checks; do not close #3492 on partial publication.

## Done when

- Final `git archive` inputs are the protected merge; CUDA has 87 and OpenTelemetry 72 members, with no generated binaries or superseded CFF date.
- Both releases are non-draft, non-prerelease, not Latest, and their tags resolve to `4daf787f3064e348d5659c4f623429aa8b5c9f30`.
- Downloaded archives match the uploaded files and SHA-256 manifests; extracted CUDA host/evidence and OpenTelemetry telemetry/default-off checks pass.
- Actual URLs and archive digests are recorded; public repo status/citation date/item move from pending to published/done only in the close PR.
- Fresh exact-head review and GitHub checks approve the close PR; #3492 closes only after those verified releases exist.

## Stop and report

Stop after publication records merge and fresh-main verification. Do not submit upstream, advertise NVIDIA/OpenTelemetry acceptance, or start new benchmarks. If either release fails read-back, report the exact partial state and keep #3492 open.
