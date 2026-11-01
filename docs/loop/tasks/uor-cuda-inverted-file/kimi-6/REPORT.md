# REPORT: make release registration claimable, kimi-6

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Order: [ORDER.md](ORDER.md). Closes #3475; References #3492.

## What did not work

- Fresh review at `6a0e01da5` found a hard claimability defect missed by policy/gates: new-item scope listed shared `docs/roadmap/README.md`, so `loop claim` would refuse the item.
- The public README still called official-collector interoperability and corrected independent replication unfinished, contradicting accepted evidence. The new item also omitted the real-loaded-model NOT_RUN and experimental local `uor.*` status.
- The direct dependency check then found #3475 still `ready`, so `loop next` would hold the release item despite merged accepted work. Removing that truthful dependency was rejected.
- Canonical sync closed #3475 from the branch's new status before protected delivery. It was immediately reopened with an explanatory comment; this PR's `Closes #3475` is the intended close point.
- Earlier review history remains unchanged. No release, implementation, measured evidence, or upstream submission changed.

## What changed

Removed the shared roadmap page from new-item scope; retained canonical `issues.json`. Added the loaded-model and experimental-attribute limits. Corrected the public README to official-collector PASS and corrected independent replication PASS while preserving SDK/multi-runtime, production-export, GPU-telemetry, and other NOT_RUN boundaries.

Recorded a top-session [status decision](DECISION.md): #3475 meets every declared acceptance criterion through merged #3489 and its fresh APPROVE records, so this PR marks it `done` and closes it. #3492 remains open for actual publication. Added this repair ORDER/REPORT; canonical sync updates both item bodies without duplication.

Repair content before this status update: three files (repair ORDER, item, README). The status decision changes the parent item and this ORDER/REPORT/DECISION only. No size cap applies.

## Measurements

After marking #3475 done, the roadmap has 125 items: ready 21, held 10, decision 1, done 93. Both release dependencies are done, and #3492 has one mapping. Harness source confirms `docs/roadmap/README.md` is one of nine shared files that every PR may change but no item scope may list. Base checks confirm the FILEMAP advisories predate this PR. No model, GPU, collector, or performance measurement was run.

## Checks run at the head

After the final text commit, run the full 11 gates, `loop_shared_files` harness tests, roadmap check/render, `git diff --check`, and local policy. Post their exact-head output on PR #3494 before fresh review. Any FAIL or new advisory blocks queueing; only the explicitly accepted new-item scope advisory and pre-existing FILEMAP advisories may remain.

## Skills used

Fresh adversarial delivery review, systematic debugging, scope guard, solution integrity, direct harness-source verification, and top-session status review of #3475 against merged #3489 and CHARTER §5.1. No model compute or paid resource was used.

## Still open

A fresh exact-head review and GitHub checks must pass before registration merges and closes completed #3475. Release metadata, actual publication, asset download/checksum verification, and #3492 closure remain later steps. No upstream acceptance or release is claimed here.

## Plain summary

The completed CUDA package item can close, the release task can be claimed, and the public documentation agrees with the collector and replication evidence. The two new releases themselves are still unpublished.
