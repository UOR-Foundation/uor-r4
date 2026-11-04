# REPORT: target point medoids, deepseek-123

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[point medoids](../../../harness/prep/cpu-training-pilot-point-medoid-2026-11-04.md).
Base: `5b3b8f361`.

## What did not work, first

The first automated claim again failed on its generated commit trailers and
rolled itself back. The branch and worktree were recreated manually. The first
manual ORDER push was refused because its item link was split across lines and
the required REPORT stub was absent; both were corrected before implementation
was published.

This is the in-progress record. No public comparison has run yet.

## What changed

Added target-level geometric medoids to the existing unseen-key point-transfer
driver without changing its raw-row artifacts. Training groups the production
point rows by supervised target and chooses an observed medoid by exact L1 sum.
The unit also builds a matched shuffled-target medoid control, evaluates both
through the complete Hamming/memory/point path and applies the pre-registered
paired opening margin plus complete teacher-forced non-loss check.

## Measurements

None yet. The pre-registration is PENDING and must be FIT before public compute.

## Checks run at the head

Focused point-transfer release tests currently pass: 16/16, including medoid
selection, distance and target ties, deterministic construction, shuffled
targets, served-query agreement, malformed tables, paired margin arithmetic and
non-zero point-fallback exposure. Strict package Clippy also passed. Final
checks are recorded after the evidence commit.

## Skills used

No system-provided skill applied.

## Still open

- FIT review and the two public model-seed runs.
- Final evidence, report, notebook/roadmap updates and exact-head delivery.

## Plain summary

This unit tests a different point-training representation: one observed
geometric medoid per target instead of the raw per-example rows and their local
vote. It has not produced public evidence yet.
