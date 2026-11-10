# REPORT: Stage 0 contract and protected baseline, kimi-1

Lab: kimi. Item: [studio-cursor-hosted](../../../../roadmap/items/studio-cursor-hosted.md) (#3917). Order: [ORDER.md](ORDER.md). Roadmap base: `993bfd36cc4ef4660d482d0f65238361d7126e12`. References #3569; leaves it open and unchanged. This is an in-progress implementation report, not a stage-completion claim.

## What did not work

- Initially, official roadmap check/render/sync failed on four errors in two unchanged base items: `float-kernel-fusion` lacked its required section and `infra-desktop-client-tauri` used invalid `in_progress`; their issue-id errors were derivative. No manual issue or patched-tree workaround was used.
- Agent-100 confirmed `--only` filters writes only after whole-corpus validation. A separate PM repair was required and delivered through PR #3916, merged as `993bfd36cc4ef4660d482d0f65238361d7126e12`; those edits do not belong to this Studio ORDER.
- Agent-101 found that the repair REPORT initially expected one new metadata key. The actual marker-aware result is two: `studio-cursor-hosted → 3917` and existing `m7-lexlean-studio-tools → 3704`. That expectation was corrected and independently approved before apply.
- The first browser build stopped before Playwright because generated `pkg/uor_r4_integer.js` was absent. This was an input-preparation failure, not a source negative. The official typed dry-run publisher generated and verified the inputs without pushing or publishing.
- A fresh-main gate invocation initially used a nonexistent absolute harness path. That was a refused launch, not a gate result. The standard local build then succeeded and the full gates passed.

## What changed

- PR #3914 registered `studio-cursor-hosted`; PR #3916 repaired the direct sync preconditions. Official `roadmap sync --only studio-cursor-hosted --apply` then created **#3917**, updated only M7 #2035's checklist, and generated `issues.json` and the roadmap page.
- This ORDER now names #3917 at the repaired base. It pins exact resume, raw-JSON duplicate-key, single-state-owner, single-send-path, skip-link and 1023/1024 behavior for Tasks 1–4.
- `issues.json` gained exactly `m7-lexlean-studio-tools → 3704` and `studio-cursor-hosted → 3917`. The render links both rows. #3704, #2826 and #3432 remain closed; #3569 remains open with unchanged acceptance. The M7 checklist gained #3704 and #3917; no other milestone issue changed.
- Studio baseline was re-established with pinned dependencies and official generated inputs: **455/455 typed tests across 52 files** and **13/13 existing shell/workflow browser checks** passed. No product source change or publication occurred.
- Prior delivery evidence and the recovery ledger were preserved before the merged registration worktree was removed. The separate repair worktree is cleaned up after its fresh-main verification.

## Measurements

- Official roadmap: **143 items OK**; `roadmap render --check` current. Issue #3917 is OPEN with `area:serving`, `roadmap:item`, milestone 28 (M7).
- Remote comparison: #3917 added; no roadmap item removed and no pre-existing roadmap item body/state/label changed. Separate milestone comparison shows only #2035 updated, adding #3704 and #3917 to its children.
- Registration verification: **33/33** roadmap unit tests and **11/11** full gates. Current baseline: **455/455** typed tests and **13/13** browser checks. These are baseline results, not numbered-task evidence.
- Task 1 added the Rust contract crate and exact `run.resumed`/raw-parser interfaces; its implementation is complete, not a model capability result. No model compute, paid service, credential, deployment or real R4 generation occurred.

## Numbered task evidence

- **Task 1 complete:** `82acb49de` added the schema/event/canonical contract after unresolved-interface RED; `5afd33535` repaired cursor continuity after a second RED. Final focused suite: **13 contract + 9 canonical tests**, exit 0. Implementer also reports full crate tests, fmt, clippy, diff checks and deterministic schema export at 10,671 bytes / SHA-256 `20079c51893c30faa9789158ca3aa3d3c7ac486bbb3c3555ce51316bda8b7ebf`.
- Independent Task 1 review found sequence-gap and resume-N+1 defects plus a weak replay test. Scoped re-review confirms all three **ADDRESSED** with no new Critical/Important breakage. Exact-next cursor checks and `check_resume` now enforce scope, generation, sequence, previous generation, digest and timestamp. `task-1-report.md`, RED/GREEN logs and both review packages are retained in the SDD workspace.
- Deferred minors remain explicit for final whole-branch review: object-only unknown data v1, payload approval/error-code taxonomy, and non-event object-schema completeness. Task 2 must match exact schema file bytes/trailing newline, lowercase control escapes and `Object.is(-0)`, and generate/template AgentError/AgentCursor/ApprovalRequest consistently rather than claiming they are in schema.
- Task 2 is now implementing deterministic TypeScript generation with cross-runtime canonical/digest assertions; it has no completion result yet.

## Checks run at the head

- Post-repair `roadmap check`: PASS (`local/kimi/cursor-issue-sync/post-repair-check.log`). Read-only sync plan: 1 create, 0 updates, 0 closes, 1 M7 checklist (`post-repair-dry-run.log`).
- Official apply: `apply.log`; #3917 direct identity: `studio-issue.json`; generated metadata diff: exactly two keys; remote/milestone comparisons: `remote-write-diff.json`, `milestones-before/`, `roadmap-issues-{before,after}.json`.
- Post-apply `roadmap check` and `roadmap render --check`: PASS (`post-apply-check.log`, `post-apply-render-check.log`).
- Baseline logs: `npm-ci.log`, `baseline-vitest.log`, `baseline-prepare.log`, `baseline-build.log`, `baseline-playwright.log` under `local/kimi/cursor-issue-sync/`.
- Task 1 RED/GREEN: `task-1-red.log`, `task-1-green.log`, `task-1-fix1-red.log`, `task-1-fix1-green.log`. Task 2+ RED/GREEN tests remain NOT_RUN. Current-head local policy/full gates and protected delivery remain pending until the stage completes.

## Skills used

- Isolated worktree and protected delivery, working discipline, solution integrity, subagent-driven implementation with fresh implementer/reviewer per task, red-first development, systematic instrument diagnosis, receiving/verifying independent reviews, scoped fix-loop re-review, and verification-before-completion.

## Still open

- Tasks 2–23 with their required red-first tests, independent task reviews, stage acceptance checks and commits. Task 1 is complete; Task 2 is in progress.
- Deferred Task 1 minors, final whole-branch triage, stage REPORT completion, exact-head approval, required GitHub checks, protected merge and fresh-main verification.
- Hosted services/integrations, real R4 generation, useful-model evaluation, production deployment and full Cursor parity remain NOT_RUN and outside this local plan.

## Plain summary

The official roadmap path and issue #3917 are complete. Task 1 now provides the Rust event schema, strict canonical JSON parser, exact cursor continuity and resume contract after independent fix review. Task 2 is generating the matching TypeScript contract. No hosted service, real R4 generation, useful model behavior or production capability is claimed.
