# REPORT: 4,096-row 19-table CPU count training, deepseek-125

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[trunk count dose](../../../harness/prep/cpu-training-pilot-trunk-count-dose-2026-11-04.md).
Base: `09c4f4abb`.

## What did not work, first

This is the in-progress record. No public comparison has run. Deepseek-124
established writeback correctness but its 128-row dose changed no held-out
decision.

## What changed

No source change is planned. This unit increases the additive training dose to
the driver's existing 4,096-row ceiling and applies the registered paired
top-1 and admitted/top-8 non-loss bar.

## Measurements

None yet. The pre-registration is PENDING and must be FIT before public compute.

## Checks run at the head

Final checks will be recorded after the run.

## Skills used

No system-provided skill applied.

## Still open

- FIT and public compute.
- Final evidence and delivery.

## Plain summary

This unit asks whether a 32-times larger additive count dose can change and
improve public held-out decisions where 128 rows did not.
