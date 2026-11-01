# REPORT: prepare versioned evidence releases, kimi-1

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Order: [ORDER.md](ORDER.md). Release-definition head: `367271c2b0452199d01b8b844eef2a5c30960295`; the only later change is this report.

## What did not work

- Neither public release exists yet. This PR prepares metadata only; tags, release objects, published-asset checksums, and #3492 completion remain pending the next order.
- The first trial CUDA archive omitted the telemetry convention linked by its README. The corrected archive includes `docs/telemetry/` and passes from extraction.
- Registration required separate repairs for merge-base policy, shared-file claimability, and evidence wording. PR #3494 is now merged with fresh APPROVE; #3475 is closed at its accepted scope and both release dependencies are done.
- The machine's unqualified `cargo` is a different CLI, and two harness invocations initially lacked the binary/lab identity. Those were invocation refusals, not gate results; corrected full runs are reported on the registration PR.
- `xtask links` completed with 23,742 checked links and reported the repository's existing 1,212 broken entries; this unit does not claim to repair that baseline. New relative release links resolve inside the archives; external release URLs become valid only at publication.
- One registration command used the owner checkout by mistake. Its exact original #3416-only roadmap diff was restored and verified; no staged or implementation change remains there.
- No GPU measurement or collector ingestion was rerun. No NVIDIA/OpenTelemetry submission, endorsement, accepted semantic convention, general-kNN result, end-to-end speedup, production telemetry, or model-capability claim is added.

## What changed

Prepared public notes and verification instructions for `cuda-inverted-file-v0.1.0` and `otel-artifact-v0.1.0`; changed the CUDA citation from `0.1.0-dev` to `0.1.0`; linked the releases from the CUDA README/outreach and local telemetry conventions; preserved official-collector PASS and independent-replication PASS with their remaining NOT_RUN scopes. The implementation and measured evidence are unchanged.

Size before this report: seven files (ORDER included), 147 insertions and 11 deletions. This report is the only additional file. All paths are inside the ORDER's documentation/release scope; no line cap applies.

## Measurements

Trial archives were produced by `git archive` at `367271c2b`, not assembled from untracked build output. CUDA has 84 members and OpenTelemetry 72; the archives are approximately 60 and 52 KiB. They contain no generated binaries, checkpoints, corpora, credentials, or scratch directories.

- CUDA extracted archive: `make telemetry-test evidence` passed; all three host logs have normalized SHA-256 `7b6f674abcef33a37753de8aac6000ff1584e983ebeb4eb35fc67bf5d45d5e7d`.
- OpenTelemetry extracted archive: `make telemetry-test instrumented-example` passed with `UOR_OTEL_OUT` unset.
- A separate empty-directory execution printed `telemetry disabled: no file or endpoint created` and left the directory empty.
- Both generated `SHA256SUMS` manifests passed `shasum -a 256 -c SHA256SUMS` before extraction. These are trial artifacts; final archives must be recreated from the protected merge commit and downloaded again.

## Checks run at the head

At `367271c2b`, all archive checks above pass. Fresh-main registration checks also pass: roadmap 125 items with both dependencies done, render current, and 7/7 shared-file tests. Earlier focused checks passed: claim wording, 220 xtask tests, 13 claim-wording tests, docs catalog with zero missing paths, and `git diff --check`.

After this final report commit, run the full 11 gates, claim wording, roadmap/render/diff checks, and local policy again. Post exact-head results and a fresh adversarial review on the PR before queueing; GitHub required checks must be green.

## Skills used

Project research/delivery workflow, working discipline, scope guard, solution integrity, context guard, isolated-worktree workflow, systematic debugging, and fresh adversarial delivery reviews. No model compute or paid resource was used.

## Still open

Merge this metadata PR, then publish both non-draft releases at the protected merge commit with `--latest=false`, attach final archives and SHA-256 manifests, download and verify them, and record the actual URLs/digests before closing #3492. Upstream outreach remains separate.

## Plain summary

The two downloads are prepared and tested, but not yet published. This PR contains instructions and metadata only; it changes no model or CUDA/telemetry behavior.
