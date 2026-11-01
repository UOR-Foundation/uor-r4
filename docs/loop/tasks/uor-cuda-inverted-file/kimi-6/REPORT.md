# REPORT: make release registration claimable, kimi-6

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Order: [ORDER.md](ORDER.md). References #3475 and #3492.

## What did not work

- Fresh review at `6a0e01da5` found a hard claimability defect missed by policy/gates: new-item scope listed shared `docs/roadmap/README.md`, so `loop claim` would refuse the item.
- The public README still called official-collector interoperability and corrected independent replication unfinished, contradicting accepted evidence. The new item also omitted the real-loaded-model NOT_RUN and experimental local `uor.*` status.
- Earlier review history remains unchanged. No release, implementation, measured evidence, or upstream submission changed.

## What changed

Removed the shared roadmap page from new-item scope; retained canonical `issues.json`. Added the loaded-model and experimental-attribute limits. Corrected the public README to official-collector PASS and corrected independent replication PASS while preserving SDK/multi-runtime, production-export, GPU-telemetry, and other NOT_RUN boundaries. Added this repair ORDER/REPORT. Canonical sync updated #3492 without duplication.

Repair content before this report: three files (repair ORDER, item, README). No size cap applies.

## Measurements

The roadmap remains at 125 items and one #3492 mapping. Harness source confirms `docs/roadmap/README.md` is one of nine shared files that every PR may change but no item scope may list. Base checks confirm the FILEMAP advisories predate this PR. No model, GPU, collector, or performance measurement was run.

## Checks run at the head

After the final text commit, run the full 11 gates, `loop_shared_files` harness tests, roadmap check/render, `git diff --check`, and local policy. Post their exact-head output on PR #3494 before fresh review. Any FAIL or new advisory blocks queueing; only the explicitly accepted new-item scope advisory and pre-existing FILEMAP advisories may remain.

## Skills used

Fresh adversarial delivery review, systematic debugging, scope guard, solution integrity, and direct harness-source verification of the shared-file rule. No model compute or paid resource was used.

## Still open

A fresh exact-head review and GitHub checks must pass before registration merges. Release metadata, actual publication, asset download/checksum verification, and #3492 closure remain later steps. #3475 and #3492 stay open here.

## Plain summary

The release task can now be claimed by the harness, and the public documentation agrees with the collector and replication evidence. Nothing is published by this repair.
