# REPORT: infra-loop-harness, codex-1 (D63 finite-probe scope)

Lab: codex. Order: [ORDER.md](ORDER.md). Base: `71d9247a87ba45588ea00a583d5321ae2a3d6082`.

## What did not work, first

- D63's "Exact arithmetic is answered" overstates the cited four-test report. The implementation is unchanged at blob `d18644c1f49b8bd77aa3470a2bc0762c09d9e511`; this task repairs no implementation or post-merge finding.
- The in-progress local policy check after the content commit failed only because this REPORT was not yet added. Its owner-path check passed using the direct standing documentation authorization recorded in the ORDER. Final-head validation is posted on the PR.

## What changed

- `212ab5038` committed this unit's ORDER and pre-edit plan/cancel-probe record first, within its own task folder.
- `84ed36aa` appended D75 as a factual clarification and qualified the matching PLAN cell. Existing decision rows are byte-identical; PLAN's closed status, date, cancellation condition and oracle PENDING text are unchanged.
- This REPORT completes the new scoped record. Size: five Markdown files (two existing documents and three task records), no code, tests or generated changes; below the ORDER's approximate 110-line estimate.

## Measurements

No new arithmetic test, model experiment or capability measurement. The four-test result is attributed to the [existing report at `9aaaf13bb`](https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6382217138). The [post-merge review of #2972](https://github.com/UOR-Foundation/uor-r4/pull/2972#issuecomment-6343617086) remains linked and unresolved by this documentation change.

## Checks run at the head

- Preservation comparisons: removing only D75 reproduces the base DECISIONS file exactly; PLAN's status/date/cancel cells and full oracle-pending suffix match the base.
- `git diff --check` and Rust claim-wording pass.
- Rust `loop gates --full --lab codex`: 11 GREEN; these are tooling checks, not model evidence.
- `xtask links` baseline: 0 new broken links (961 pre-existing at the pre-edit baseline).
- `xtask archive-verify`: 0 regressions; 10,806 archive files, 0 modified or deleted.
- Roadmap check: 91 items OK. Roadmap render, catalog, five computed figures and four charts are current, without generated-file changes.
- `loop report --check` and `loop pr-check --local` final verdicts, exact-head adversarial review and CI are recorded on the PR. The owner-path gate checks the recorded direct authorization; no approval or review is fabricated.

## Skills used

`using-git-worktrees` for isolation; `verification-before-completion` for evidence before claims; `requesting-code-review` for a fresh-context adversarial review at the published head.

## Still open

#2972's arithmetic, causal-control and other post-merge findings. The registered address-as-read oracle remains PENDING with its listed prerequisites. No experiment bar, milestone, run readiness, source implementation or owner policy changes here. No issue sync, model compute, spend, branch deletion or manual deployment is part of this unit.

## Plain summary

Four existing tests were reported passing. That closes the already-executed finite probe, while broader arithmetic correctness and the published review findings remain separate. The current documents now say exactly that, with the historical decision and research state preserved.
