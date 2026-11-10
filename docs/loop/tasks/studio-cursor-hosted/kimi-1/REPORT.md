# REPORT: Stage 0 contract and protected baseline, kimi-1

Lab: kimi. Item: [studio-cursor-hosted](../../../../roadmap/items/studio-cursor-hosted.md). Issue: pending official sync. Order: [ORDER.md](ORDER.md). Base: `4265dc3d6c94015ba97bcb1505ed8d57e339a39a`. References #3569; leaves it open and unchanged. This is an in-progress draft report, not a stage-completion claim.

## What did not work

- Fresh-main `cargo run -q -p xtask -- roadmap check` fails on exactly four unchanged base errors: `float-kernel-fusion.md` lacks `## What is missing`; `infra-desktop-client-tauri.md` uses `status: in_progress`; their two `issues.json` ids have no valid parsed item.
- `cargo run -q -p xtask -- roadmap sync --only studio-cursor-hosted` fails on the same loader errors before any write. No issue number, generated metadata or manual workaround exists. The official issue and all product tasks remain pending.
- A fresh-main gate invocation initially used a nonexistent absolute harness path. That was a REFUSED launch, not a gate result. The standard `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness` succeeded and the freshly built binary then ran the full gates.

## What changed

- PR #3914 merged the new item as `4265dc3d6c94015ba97bcb1505ed8d57e339a39a`; fresh-main blobs match reviewed head `e8941d64384e5142900901e6df1a332bb450bc8b`. #3569 remains open and unchanged.
- Added this new-item Stage 0 ORDER at the merged base, including exact resume, raw-JSON duplicate-key, single-state-owner, single-send-path, skip-link and 1023/1024 rulings. No product code or generated output changed.
- Preserved the prior evidence and recovery ledger outside the removed registration worktree; deleted only that merged local branch/worktree. GitHub had already removed its remote branch.

## Measurements

- Registration fresh-main unit tests: **33/33 PASS** (`local/kimi/cursor-issue-sync/fresh-unit.log`). Full gates: **11/11 GREEN** with the newly built harness (`local/kimi/cursor-issue-sync/fresh-gates.log`). These cover the merged registration, not unwritten Stage 0 code.
- Product implementation size so far: **0 code/test files, 0 product tests**. No model compute, paid service, credential, deployment or real R4 generation occurred.

## Checks run at the head

- `cargo run -q -p xtask -- roadmap check`: **FAIL**, exact four base errors above.
- `cargo run -q -p xtask -- roadmap sync --only studio-cursor-hosted`: **FAIL before write**, same four errors.
- Registration fresh-main source/tree comparison: PASS; three intended files only. #3569 state/body acceptance remains unchanged.
- This ORDER/REPORT draft's local policy result is recorded after execution. All numbered-task RED/GREEN checks are NOT_RUN.

## Skills used

- Isolated worktree and protected branch delivery, working discipline, solution integrity, subagent-driven planning, systematic instrument diagnosis, receiving/verifying independent review, and verification-before-completion. No implementation subagent has been dispatched.

## Still open

- Official issue sync and binding the observed issue number in this ORDER; then Tasks 1–23 with their own red-first tests and independent reviews.
- Stage 0 implementation, stage REPORT completion, exact-head review, required GitHub checks, protected merge and fresh-main verification.
- Hosted services/integrations, real R4 generation, useful-model evaluation, production deployment and full Cursor parity remain NOT_RUN and outside this local plan.

## Plain summary

The separate browser-owned Studio item is registered and verified on fresh main. This draft has only its Stage 0 ORDER and an honest blocker report. The official roadmap tool cannot create the issue while two unrelated base items fail validation; no bypass is used. No product capability has been implemented or claimed yet.
