# REPORT: register community evidence releases, kimi-4

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Order: [ORDER.md](ORDER.md). References #3475 and #3492.

## What did not work

- The combined release PR's local policy check showed that a new roadmap item cannot serve as its own merge-base item. This separate registration PR addresses that root cause; it contains no release metadata and does not mark #3492 done.
- No actual release, tag, asset, implementation change, or upstream submission is delivered here.

## What changed

Registered `uor-community-evidence-releases` as a `ready` research/tooling item depending on the completed CUDA and collector-evidence lines. Mapped the existing canonical issue #3492 and rendered the roadmap. Before this report: four files, 84 insertions and two deletions; this report is the only additional file. No size cap applies.

## Measurements

`cargo run -q --locked -p xtask -- roadmap check` reports 125 valid items: ready 22, held 10, decision 1, done 92. `roadmap render --check` confirms the generated page is current. These are bookkeeping counts, not model or release-performance evidence.

## Checks run at the head

- `uor-harness loop gates --full` with `UOR_LAB=kimi`: 11/11 GREEN, including claim wording.
- `xtask roadmap check`: PASS; `roadmap render --check`: current.
- `git diff --check origin/main...HEAD`: PASS.
- Local policy and exact-head CI/review are required before queueing; results after this report commit are recorded on the PR.

## Skills used

Project research/delivery workflow, working discipline, scope guard, solution integrity, context guard, isolated-worktree workflow, and systematic debugging of the merge-base policy failure. No model compute or paid resource was used.

## Still open

Deliver the already tested release definitions in the next PR, then create both non-draft `0.1.0` releases from that protected merge commit and verify downloaded archives/checksums. Close #3492 only after publication and read-back verification.

## Plain summary

This PR only creates the roadmap entry for the two planned downloads. It does not publish anything, close the release task, or change model, CUDA, or telemetry behavior.
