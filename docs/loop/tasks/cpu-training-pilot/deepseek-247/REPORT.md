# REPORT: cpu-training-pilot, deepseek-247

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `uor-harness loop claim cpu-training-pilot --lab-topic real-ask-regression`
  refused on the known umbrella-scope false overlap with an unrelated open
  one-model-end-to-end order. This unit uses a hand-written ORDER and leaves
  the policy gate to prove its actual changed-path scope.
- The first push was blocked because this REPORT did not yet exist at the
  head. The branch remains local; no work or external state was lost.

## What changed

Implementation is in progress. The ORDER is committed and this REPORT stub
exists so the branch can satisfy the loop's commit-order policy. No product
code, measurement or claim has landed yet.

## Checks run at the head

None yet beyond the pre-push policy run that reported the missing REPORT.

## Still open

All ORDER done-when items remain open.

## Plain summary

This branch will turn the already-measured real checkpoint audit into a
repeatable CPU regression. No model, GPU, serving path or Claude-owned file is
being changed.
