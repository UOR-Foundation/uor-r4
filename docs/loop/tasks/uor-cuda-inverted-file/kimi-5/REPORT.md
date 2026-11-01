# REPORT: repair community evidence release registration, kimi-5

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Order: [ORDER.md](ORDER.md). References #3475 and #3492.

## What did not work

- Fresh review at `1d710e969` returned CHANGES: the expected new-item scope advisory lacked an explicit top-session decision, the gate ledger predated the report commit, "completed CUDA line" overstated still-open #3475, and unqualified interoperability wording could be read as SDK/multi-runtime coverage.
- No generated-roadmap, dependency, duplicate-issue, implementation, evidence, or release-status defect was found by that review. No release or upstream submission exists in this PR.

## What changed

Added this repair ORDER; limited the new item's evidence wording to official-collector interoperability and preserved SDK/multi-runtime NOT_RUN; recorded the narrow D76 scope decision in the repair ORDER. Canonical sync updated #3492 in place and created no issue. This report replaces the earlier "completed lines" wording by keeping #3475 open at its declared evidence scope.

Repair content before this report: two files (the repair ORDER and item wording). No size cap applies.

## Measurements

The registered item count remains 125, with existing #3492 mapped once. No model, GPU, collector, performance, or release-asset measurement was run; this is a documentation/policy repair only.

## Checks run at the head

The final text commit is followed by a fresh 11-gate run, `roadmap check`, `roadmap render --check`, `git diff --check`, and `loop pr-check --local`. Their exact-head results are posted on PR #3494 before the new review and queue decision. Only the explicitly reviewed canonical-new-item scope advisory and known FILEMAP advisories are accepted; any FAIL blocks queueing.

## Skills used

Fresh adversarial delivery review, systematic debugging, scope guard, solution integrity, and top-session review of CHARTER §5.3/§5.10 and D76. No model compute or paid resource was used.

## Still open

A fresh exact-head review and GitHub checks must pass before this registration merges. The later metadata PR and actual releases remain separate; #3475 and #3492 stay open and the PR uses References only.

## Plain summary

The review asked for clearer evidence limits and checks tied to the final commit. Those corrections are made without changing the release design, implementation, or measured evidence.
