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
memory-off and trained-memory-off. Three direct-target arms use the stored
memory target at an exact key hit and the exact CPU writer only on a miss:
frozen, trained-fallback and trained-fallback with a deterministic
shuffled-target control.

## Measurements

**Model replay NOT RUN.** The public seed replay is blocked by the missing
non-author FIT. No model decision, generated reply or throughput reading is
reported here.

The model-free selection census is complete. The 114 evaluation references
hold 1,326 tokens after one trailing BOS is removed. The longest is 36 tokens.
**104 of 114** references fit the registered 16-token reply cap and 192-token
context; ten references of 26–36 tokens are unreachable under the fixed cap.
This means 104 prompts can still be judged for exact complete reply once FIT
clears the replay.

The training-only key census is stronger: **114/114** evaluation opening keys
hit the derived memory table and **50** stored majority targets are correct.
For reply rows, **795/1,212** keys hit and **717** stored majority targets are
already correct, 59.2% of all evaluation reply decisions. Only 44 of 724 reply
keys, covering 133 of 1,192 training reply decisions, are ambiguous. This is a
table-content ceiling, not served-model output, but it shows that table absence
and label ambiguity do not explain deepseek-109's low served top-1.

The pinned census is [reference-census.json](evidence/reference-census.json).

## Checks run at the head

- Seven focused reply-evaluator tests passed, including deterministic
  query-movement and direct-target replies over exported synthetic CPU
  artifacts.
- Full CPU release suite: 171 library tests passed, 4 release benchmarks
  ignored; every binary suite passed.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- `uor-harness prep-check`: sheet is complete but `PENDING`, not cleared for
  compute.
- Model-free `--select-only --reply-cap 16` reproduced 16 keys, 102 training
  examples, 114 evaluation examples and the 104-reachable-reference census.

## Skills used

The repository's `experiment-prereg` instructions governed the pending sheet.
No system-provided skill applied.

## Reading

The instrument is ready; the model question is unanswered. A future fresh
non-author FIT can authorize the bounded replay. Until then, no claim is made
about whether trained movement weights affect complete replies or termination.

The key census changes the next trainer decision. The table already contains a
correct majority target for 59.2% of teacher-forced held-out reply decisions,
while deepseek-109's served model named only 12.5% correctly. Increasing the
coordinate sweep would therefore optimize the wrong bottleneck. If the pending
replay is negative, the next bounded unit should compare the current
query-moving memory read against a lawful exact-memory target arbitration that
uses the stored row directly, with the same target-shaped and shuffled-target
controls. That is a new mechanism hypothesis, not a wider version of the closed
coordinate sweep.

## Still open

- Fresh non-author TEST FITNESS for the pending sheet.
- Two-seed replay and seed-1 repeat.
- Comparison against frozen, random, memory-off and trained-memory-off.
- A materially different credit rule if complete-reply propagation is negative.
- Exact-memory target arbitration versus query-only movement as the next
  mechanism comparison.
- Complete replies, fresh phrasing, useful chat and GPU qualification.

## Plain summary

The source can now replay deepseek-109 and grade five matched artifacts on
bounded complete replies. Everything finite is implemented and green. No public
run was performed, so this unit contributes a ready instrument and a pending
pre-registration rather than a model result.
