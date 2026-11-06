# REPORT: cpu-training-pilot, deepseek-219

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[prefix-length sheet](../../../../harness/prep/cpu-training-pilot-prefix-length-2026-11-06.md).
Base: `86996114f`.

## Status

IN PROGRESS. TEST FITNESS: FIT and pre-run GO recorded at implementation
`f5cb4f7a9`; fresh seed 2782 has not been generated.

## What did not work

Not yet measured.

## What changed

Registered retrieved-prefix continuation with neighbor-length completion.
The existing experiment binary now has an opt-in length mode with model-only
cap, model-only retrieved-length, retrieved-prefix length, shuffled-prefix
length, full-retrieval and reference-length-oracle arms. Length and BOS stops
are counted separately, and length-stopped token identity can count as exact.

## Measurements

Not yet run.

## Checks run at the head

- seed-2782 panel and output roots were absent at registration.
- focused length-completion tests: 4 passed;
- full `uor-r4-cpu-train` library: 399 passed, 4 expected ignores;
- scoped Clippy passed for the changed binary with the pre-existing library
  `type_complexity` lint exempted;
- scoped rustfmt and `git diff --check` passed;
- source and release binary hashes frozen before GO.

## Skills used

No system-provided Codex skill applied.

## Still open

- fresh seed-2782 result, sealed evidence and delivery.

## Plain summary

This is the in-progress record for deepseek-219.
