# REPORT: cpu-training-pilot, deepseek-223

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[content-length sheet](../../../../harness/prep/cpu-training-pilot-content-length-2026-11-06.md).
Base: `bbb8c5332`.

## Status

IN PROGRESS. TEST FITNESS: FIT and pre-run GO recorded at implementation
`74793da76`; seed 2783 and the public runs have not started.

## What did not work

Not yet measured.

## What changed

Registered full-sequence suffix content with reference-independent stored-length
completion and matched controls on fresh seed 2783.

The evaluator now supports an exact stored-length endpoint without changing the
default boundary mode. Length completion is recorded separately from BOS, and
exact-reference identity is independent of which legal stop ended the row.

## Measurements

Not yet run.

## Checks run at the head

- fresh panel and output roots absent at registration.
- focused stored-length tests: 2 passed;
- driver usage test: 1 passed;
- full `uor-r4-cpu-train` library: 404 passed, 4 expected ignores;
- scoped Clippy passed with the pre-existing `type_complexity` lint exempted;
- changed-file rustfmt and `git diff --check` passed;
- source, driver and release binary hashes frozen before GO.

## Skills used

No system-provided Codex skill applied.

## Still open

- implementation, FIT/GO, fresh result and delivery.

## Plain summary

This is the in-progress record for deepseek-223.
