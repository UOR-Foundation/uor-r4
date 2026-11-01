# REPORT: publish and verify both evidence releases, kimi-4

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Order: [ORDER.md](ORDER.md). Publication-record head: `3d2407e094ab69aad41e342224cd33adfeea128d`; only this report follows.

## What did not work

- Release creation alone was not accepted as completion: both published archives had to be downloaded and pass byte comparison, manifests, extracted tests, and default-off execution. Those checks are now complete.
- `roadmap sync --apply` closed #3492 from the branch's done status before the close PR merged. It was immediately reopened with an explanatory comment; this PR's `Closes #3492` is the intended close point.
- No NVIDIA or OpenTelemetry submission, endorsement, accepted convention, general-kNN/end-to-end-speedup claim, production telemetry, or model-capability claim was added. No GPU or collector run was repeated.

## What changed

Created two actual non-draft, non-prerelease GitHub releases with `--latest=false`, targeting protected `main` merge `4daf787f3064e348d5659c4f623429aa8b5c9f30`. Attached final source/evidence archives and SHA-256 manifests. Updated public repository text from pending to the verified release URLs, restored the citation release date, marked #3492 done, and rendered the roadmap. This PR closes #3492.

Publication records before this report: ORDER plus seven public documents and the generated roadmap page. No size cap applies; no implementation or measured evidence changed.

## Measurements

| Release | Archive SHA-256 | Members | Bytes |
|---|---|---:|---:|
| [`cuda-inverted-file-v0.1.0`](https://github.com/UOR-Foundation/uor-r4/releases/tag/cuda-inverted-file-v0.1.0) | `dff4c9043385abbdc36d388593d35168ff6843227b2e8064ffcfd3450ce9307b` | 87 | 60,834 |
| [`otel-artifact-v0.1.0`](https://github.com/UOR-Foundation/uor-r4/releases/tag/otel-artifact-v0.1.0) | `e7175c276bb6fc7c350d5ca242f53d1a4e5e7f0b1e74dd4b43caf073185d8fed` | 72 | 52,782 |

Both remote tags resolve to `4daf787f3064e348d5659c4f623429aa8b5c9f30`. GitHub reports `isDraft=false` and `isPrerelease=false` for both, and the historical `uor-r4 v0.1` remains the only Latest release.

Downloaded archives byte-match the uploaded files and the API-reported digests above. Both `SHA256SUMS` checks pass. The CUDA retained-record roadmap target is present. CUDA extracted `make telemetry-test evidence` passes with normalized host-log SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`. OpenTelemetry extracted `make telemetry-test instrumented-example` passes with `UOR_OTEL_OUT` unset, and a separate empty-directory run leaves it empty.

## Checks run at the head

Publication/read-back checks above ran on assets from protected merge `4daf787f`. After this final report commit, run full 11 gates, claim wording, roadmap/render/diff checks, and local policy; post results on the close PR before fresh review and queueing. The close PR changes documentation/status only.

## Skills used

Protected release delivery, exact archive/API/download verification, solution integrity, scope guard, and fresh adversarial review of the pending definitions. No model compute or paid resource was used.

## Still open

The close PR needs fresh exact-head review, green GitHub checks, protected merge, and fresh-main verification. Upstream outreach remains a separate decision; publication does not request or imply it.

## Plain summary

Both real releases are public. Their downloaded archives match the published checksums and pass the same host and telemetry checks as the reviewed trial bundles. This PR records those facts and closes the release task.
