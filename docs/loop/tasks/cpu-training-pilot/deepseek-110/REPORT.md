# REPORT: cpu-training-pilot, deepseek-110 (memory-weight complete replies)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[pending memory replies](../../../harness/prep/cpu-training-pilot-memory-replies-2026-11-03.md).
Base: `5707380fe`.

## What did not work, first

- **No public result exists.** The pre-registration is `TEST FITNESS: PENDING`
  because no fresh non-author reviewer recorded a FIT. Running the registered
  public replay before that would repeat the project's L-012 failure.
- **This is not a new trainer.** It replays deepseek-109's exact coordinate
  descent to determine whether that mechanism propagates into complete replies.
  It does not reopen the failed rank-margin family or widen the coordinate
  sweep.
- **The point-vote complete-reply result does not answer this question.** The
  point-vote artifacts had zero movement weights; deepseek-109 trained opening
  and reply movement weights and was graded only token by token.

## What changed

Added a bounded complete-reply evaluator to `uor-r4-cpu-train`:

- every prompt token is stepped once;
- reply start is marked before the final prompt token;
- generated tokens are fed back until shared integer BOS, cap or context stop;
- one trailing BOS is removed from the reference;
- exact-reference, reachability, opening, distinct-text, repeated-4-gram,
  generated-token and digest counters are deterministic.

`cpu-memory-panel-driver` now accepts:

- `--reply-cap N`, which adds five matched complete-reply readings;
- `--expected-trained-sha256 HEX`, which refuses artifact drift before outputs
  are used.

The arms are frozen memory, trained memory, equal-accepted-move random,
memory-off and trained-memory-off.

## Measurements

**NOT RUN.** The public seed replay is blocked by the missing non-author FIT.
No artifact, prompt, reply or throughput reading is reported here.

## Checks run at the head

- Five focused reply-evaluator tests passed.
- Full CPU release suite: 168 library tests passed, 4 release benchmarks
  ignored; every binary suite passed.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- `uor-harness prep-check`: sheet is complete but `PENDING`, not cleared for
  compute.

## Skills used

The repository's `experiment-prereg` instructions governed the pending sheet.
No system-provided skill applied.

## Reading

The instrument is ready; the model question is unanswered. A future fresh
non-author FIT can authorize the bounded replay. Until then, no claim is made
about whether trained movement weights affect complete replies or termination.

## Still open

- Fresh non-author TEST FITNESS for the pending sheet.
- Two-seed replay and seed-1 repeat.
- Comparison against frozen, random, memory-off and trained-memory-off.
- A materially different credit rule if complete-reply propagation is negative.
- Complete replies, fresh phrasing, useful chat and GPU qualification.

## Plain summary

The source can now replay deepseek-109 and grade five matched artifacts on
bounded complete replies. Everything finite is implemented and green. No public
run was performed, so this unit contributes a ready instrument and a pending
pre-registration rather than a model result.
