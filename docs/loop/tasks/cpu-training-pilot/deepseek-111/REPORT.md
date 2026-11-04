# REPORT: cpu-training-pilot, deepseek-111 (reply termination diagnostic)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[pending termination diagnostic](../../../harness/prep/cpu-training-pilot-reply-termination-2026-11-04.md).
Base: `3f7f8605b`.

## What did not work, first

- **No public diagnostic exists yet.** The sheet is pending a fresh non-author
  FIT. No generated step has been inspected.
- **The deepseek-110 negative does not identify the termination failure.**
  Zero BOS stops could mean BOS was absent from admission, admitted but ranked
  below another token, or ranked first and suppressed by the decoder. This unit
  separates those cases.
- **This is not another memory training run.** It replays the pinned artifacts
  only to explain the writer's BOS treatment.

## What changed

The CPU FBS forward can now diagnose a target token's writer decision without
mutating session state:

- target cell and admission;
- one-based rank among admitted writer candidates;
- top-8 and top-1 status;
- target/chosen L1 distances and their difference.

The bounded reply evaluator can emit this diagnostic at every generated step
while preserving the generated reply sequence and digest.

## Measurements

**NOT RUN.** Public compute is blocked pending FIT.

## Checks run at the head

- Nine focused reply-evaluator tests passed.
- The writer diagnostic test passed.
- Full CPU release suite: 174 library tests passed, 4 release benchmarks
  ignored; every binary suite passed.
- Strict all-target CPU Clippy and formatting passed.
- No public model inference was run.

## Skills used

The repository's `experiment-prereg` instructions governed the pending sheet.
No system-provided skill applied.

## Reading

The instrument is ready; the failure class is unknown.

## Still open

- Fresh non-author FIT.
- Two-seed diagnostic replay and seed-1 repeat.
- Classification and next mechanism.
- Complete replies, fresh phrasing, useful chat and GPU qualification.

## Plain summary

The decoder now records whether BOS is admitted and how it ranks at every
generated step. No public result exists yet.
