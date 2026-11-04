# REPORT: within-cell orbit termination, deepseek-122

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `cb457faa4`.

## What did not work, first

- The automated `loop claim` failed while creating its initial commit because
  the generated message lacked the required `Lab:` and `Order:` trailers. It
  reported a complete rollback. The ORDER and claim were recreated manually.
- A point-key context repair was refused because Claude's open
  `one-model-end-to-end` order shares the `stack/format.rs` scope. That repair
  is not bypassed here.

## What changed

This is the in-progress record for the within-cell orbit-signature termination
experiment. The ORDER is committed before implementation and compute.

## Measurements

None yet.

## Checks run at the head

The final checks will be recorded before review.

## Skills used

No available Codex skill applied.

## Still open

- Implement and FIT-review the orbit-signature gate.
- Run the registered two-seed comparison and update this report.

## Plain summary

This unit tests whether rows that share the same writer cell can still be
separated by their relative position among the 16 orbit roots. It has not yet
produced a measurement.
