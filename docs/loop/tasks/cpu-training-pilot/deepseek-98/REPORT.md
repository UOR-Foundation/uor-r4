# REPORT: cpu-training-pilot, deepseek-98

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

Implementation is in progress. The automated claim failed on a commit-hook
trailer mismatch and rolled back cleanly; this draft was created by hand.

## What changed

This draft records the deterministic parallel panel/counterfactual unit before
implementation. No source change is claimed yet.

## Measurements

The base panel benchmark measured 10,836.104 rows/s over 1,024 timed rows on
one thread. No post-change timing is claimed yet.

## Checks run at the head

The base release panel benchmark completed with zero query, token and kept-cell
mismatches.

## Skills used

No available Codex skill applied.

## Still open

Implementation, exact serial/parallel equivalence tests and release benchmarks
remain open in this unit.

## Plain summary

This draft will make independent CPU evaluation work use the available cores
while keeping the integer results and update order unchanged.
