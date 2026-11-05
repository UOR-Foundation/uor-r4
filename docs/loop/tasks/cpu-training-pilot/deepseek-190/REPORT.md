# REPORT: cpu-training-pilot, deepseek-190

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `3ac40d6d5`.

## Status

In progress. The exact scale-serving preflight is claimed before implementation
or compute.

## What did not work

The automated `loop claim` rolled back because its generated commit lacked the
required `Lab:` and `Order:` trailers. This unit uses the documented hand-claim
path with the corrected trailers.

## What changed

No model or runtime code has changed yet.

## Checks run

- ORDER claim check: pass for order, scope, banned presence, owner paths and
  rule sources;
- repository policy rejected the first push until this in-progress REPORT was
  present.

## Still open

Benchmark implementation, focused tests, FIT, D67 GO, the 18-cell run,
evidence, reviewed delivery and fresh-main verification.
