# REPORT: repair public release definitions, kimi-2

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Order: [ORDER.md](ORDER.md). Repaired release-definition head: `710d0c62df6790309906b73a6d8b3be2116bfdd2`; only this report follows.

## What did not work

- Fresh review at `2bd8976c3` rejected premature published/downloadable statements before either release existed, a CUDA archive link to an omitted roadmap item, and unlabeled historical telemetry NOT_RUN evidence.
- The first corrected-count expectation was not used: `git archive` also includes directory entries. The actual rebuilt CUDA archive has 87 members, not 84/85; OpenTelemetry has 72.
- No release, tag, implementation, measurement, or upstream submission exists in this PR. #3492 remains open and all public repository text stays pending until actual publication/read-back.

## What changed

Changed public release text to prepared/pending wording and removed live-release URL claims. Added the roadmap item linked by retained kimi-4/5/6 records to the CUDA archive inputs and release description. Labeled `collector-results.json` as the later official-collector PASS and `evidence.json`/`delivery-checks.txt` as earlier host-only history, including their NOT_RUN/124-item scope. Preserved all measured evidence and non-claims.

Repair content before this report: repair ORDER plus five public documents. No size cap applies.

## Measurements

Rebuilt both archives at `710d0c62d` using `git archive`: CUDA 87 members, OpenTelemetry 72, approximately 60/52 KiB. Both SHA-256 manifests pass. The retained-record roadmap target and telemetry convention are present in the extracted CUDA archive.

- CUDA extracted archive: `make telemetry-test evidence` PASS; three normalized host logs have SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`.
- OpenTelemetry extracted archive: `make telemetry-test instrumented-example` PASS with `UOR_OTEL_OUT` unset; a separate empty-directory run left it empty.
- Source identity probe: `git diff --exit-code 719a9a7861deb595dfc5269ffec8a5e7e53cb39b..HEAD -- projects/uor-cuda-inverted-file/{src,tests,include,Makefile}` is empty.
- Public-text probe found no premature `published as`, `downloadable source-and-evidence`, live-tag download, or `is public after` wording in the changed release entrypoints.

## Checks run at the head

Archive and source checks above ran at `710d0c62d`. After this final report commit, run the full 11 gates, claim wording, roadmap/render/diff checks, and local policy; post exact-head results on PR #3495 before fresh review and queueing. No GPU or collector run is inferred.

## Skills used

Fresh adversarial delivery review, systematic debugging, solution integrity, scope guard, and direct archive/source verification. No model compute or paid resource was used.

## Still open

A fresh review and GitHub checks must approve the repaired metadata before merge. Actual releases will use separate final notes and archives from the protected merge commit, then download/checksum verification and a close PR. #3492 stays open.

## Plain summary

The repository now describes two prepared releases without pretending they exist. Their downloads include the files linked by retained records and clearly separate old host-only evidence from the later collector success.
