# REPORT: m1-close-saved-model, kimi-51

Lab: kimi. Order: [ORDER.md](ORDER.md).

References #2837 and #3319. Host-only instrument repair; no milestone or issue closes.

## What did not work, first

Harness CI on the render PR failed `preprules::tests::the_real_sheets_pass_the_shape_rule`: the M1-close prep sheet lacked Base artifact, Data split and Steps blocks. The executed M1 result was not implicated.

## What changed

The existing checkpoint/control content is under Base artifact, the pinned corpus under Data split, and the GPU protocol under Steps. Cost/power and verdict are renumbered consistently. No result, bar, threshold, evidence, code or issue state changed.

## Measurements

No experimental measurement. `cargo test --locked -p uor-r4-harness the_real_sheets -- --nocapture`: 3 passed, covering the formerly failing shape rule and both real-sheet readers.

## Checks run at the head

- Real-sheet tests: 3 passed.
- `git diff --check`: PASS; only heading/section changes.
- Full harness, loop gates, local policy, report check, fresh exact-head review, required checks, queue and fresh-main verification are required.

## Skills used

- **systematic-debugging:** read the exact CI failure and repaired the heading shape rather than weakening the test.
- **solution-integrity:** preserved the executed M1 evidence and kept this separate from the render PR.

## Still open

- Merge this repair forward into #3319 and rerun its harness CI.
- The closed-#2820/ready-item mismatch remains separate.

## Plain summary

The prep sheet now uses the section names the harness requires. Nothing about the completed M1 experiment changed.
