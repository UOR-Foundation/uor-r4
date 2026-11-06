# REPORT: cpu-training-pilot, deepseek-221

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[nearest-rival sheet](../../../../harness/prep/cpu-training-pilot-nearest-rival-gain-2026-11-06.md).
Base: `4fde31439`.

## Status

IN PROGRESS. TEST FITNESS: FIT and pre-run GO recorded at implementation
`2ba8b56b6`; the public CPU runs have not started.

## What did not work

Not yet measured.

## What changed

Added an exact eight-rival final-gain trainer and a dedicated driver. The
trainer refreshes rival identities after each step, evaluates every gain
coordinate at both signs through the exact CPU FBS query, and uses a cyclic
target-label shuffle that preserves the target multiset. The driver starts
from deepseek-97's frozen memory baselines and includes frozen, random,
shuffled-target and memory-off controls.

## Measurements

Not yet run.

## Checks run at the head

- focused nearest-rival tests: 3 passed;
- full `uor-r4-cpu-train` library: 402 passed, 4 expected ignores;
- scoped Clippy passed for the changed binary with the pre-existing
  `type_complexity` lint exempted;
- changed trainer and driver formatting plus `git diff --check` passed;
- source, driver, baseline, corpus and output-root absence were verified
  before GO.

## Skills used

No system-provided Codex skill applied.

## Still open

- two-seed public comparison, sealed evidence and delivery.

## Plain summary

This is the in-progress record for deepseek-221.
