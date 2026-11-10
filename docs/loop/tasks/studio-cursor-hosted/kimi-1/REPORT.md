# REPORT: establish the browser-owned Studio item, kimi-1

Lab: kimi. Item: [studio-cursor-hosted](../../../../roadmap/items/studio-cursor-hosted.md); official issue sync is pending. Order: [ORDER.md](ORDER.md). Base: `d4f1e3470ed2681847f33028de3258b5053b63f8`. Draft PR: #3914. References #3569; does not close or modify it.

## What did not work

- The official `cargo run -p xtask -- roadmap check` and `roadmap render` commands are RED on four errors in two unchanged base items: `float-kernel-fusion.md` lacks `## What is missing`; `infra-desktop-client-tauri.md` uses `status: in_progress`; consequently their two `issues.json` ids have no valid parsed item. These files and `issues.json` match `origin/main` byte-for-byte. The 11 harness gates previously passed but do not execute this validator, so that result is not evidence of roadmap validity.
- Official issue sync also calls the same checked loader and cannot run until that base condition changes. No manual issue creation, generated-file edit, validator weakening, unrelated item repair, or alternate render path was substituted.
- No application, model, API, service, browser or Tauri work has been performed in this unit. Full repository-check, independent-review and protected-delivery results are recorded only when available.

## What changed

- Added `studio-cursor-hosted` as a separate ready, host-checked M7 item with browser-owned task/runtime acceptance, optional tool-only Tauri execution, IndexedDB persistence, review/editor workflows, and explicit local acceptance limits.
- Preserved #3569 and its same-Rust-runtime acceptance. This item neither satisfies nor closes it.
- Added binding rulings for a real `ComposerSurface`, one typed task list/send path, independent 1023/1024 drawer behavior, mode-specific skip links, transport experimentation, native approval authority, explicit `run.resumed`, truthful empty states, provider-key exception, quota/lease behavior, canonical argument rejection and serial red-first task reviews.
- Scope is two new files plus this REPORT; no existing source or generated file changed. ORDER commit: `23f936db3`; item commit: `117b642b3`. Size is reported, not capped.

## Measurements

- New item: 55 lines. ORDER: 3,848 bytes, below the 4 KB read-path guidance. This REPORT is the only file in its follow-up commit; final HEAD/tree are bound in the PR and local evidence after checks rather than guessed here.
- Disk at probe: 34.2 GiB free. No model artifact, paid compute, credential or deployment was used.

## Checks run at the head

- `cargo test -q -p xtask roadmap::tests`: **33/33 PASS**; these are validator unit tests, not whole-roadmap validation.
- `git diff --check`: PASS before the item commits.
- Base-identity comparison: #3569, the two malformed roadmap items and `issues.json` match `origin/main`; no diff.
- `cargo run -q -p xtask -- roadmap check` and `roadmap render`: **FAIL**, four pre-existing errors listed above; no output generated.
- Broader `uor-harness check`, loop report/policy/full gates and GitHub checks: see the retained final logs when complete; pending results are not PASS.

## Skills used

- Isolated-worktree verification, working discipline, solution integrity, writing-plans correction review, subagent-driven task preparation, systematic root-cause investigation of the validator failure, and protected delivery.
- Independent delivery review is a separate fresh agent; no subagent decided the new item's acceptance or scope.

## Still open

- The official roadmap validator/renderer and issue sync remain blocked by the unchanged base records. This unit does not claim those checks passed or the new item is implemented.
- Independent exact-head approval, required GitHub checks, actual protected merge, fresh-main verification and cleanup are pending until observed.
- All 23 implementation tasks remain pending. Hosted service/integrations, real R4 generation, useful-model evaluation, production deployment and full Cursor parity are NOT_RUN and outside this item's local implementation authorization.

## Plain summary

This is the acceptance and implementation-order unit, not a product capability delivery. It creates a separate browser-owned Studio item and leaves the older #3569 acceptance untouched. The new documents are committed in draft PR #3914. The roadmap validator and renderer currently fail on two pre-existing malformed items; that failure is preserved rather than hidden or repaired out of scope. No model, hosted service, production release or full-parity claim is made.
