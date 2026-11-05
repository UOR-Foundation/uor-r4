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

## Measurements

Not run yet.

## Checks run at the head

- ORDER claim check: pass for order, scope, banned presence, owner paths and
  rule sources;
- the first push was refused until this in-progress REPORT existed and carried
  the required REPORT sections.

## Skills used

No system-provided Codex skill applies. Repository loop, claim and delivery
procedures are being used.

## Still open

Benchmark implementation, focused tests, FIT, D67 GO, the 18-cell run,
evidence, reviewed delivery and fresh-main verification.

## Plain summary

This claim is open. No benchmark, model or serving result exists yet.
