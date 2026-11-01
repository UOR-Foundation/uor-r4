# REPORT: rebase, review, and rescope the typed Studio design, kimi-1

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440, completed parent). New item: `studio-typed-shell-upgrade` (#3500). Order: [ORDER.md](ORDER.md). Rescoped head: `49a39b52e832a2115835758bbd6bbd20b7f4901a`; only this report follows.

## What did not work

- The original design incorrectly assumed `model_sources.js` exposed no byte progress. Its downloader does emit real bytes/totals, but no public progress channel reaches a new store. The original all-indeterminate model and byte-identical-file constraint were therefore unimplementable without DOM scraping or duplicated loading.
- The independent builder check found four more contract gaps: a live `modelSources.v1` writer conflicts with a copy-to-new-key/no-dual-write rule; failed replacement can leave the previous model able to receive a prompt; picker-derived privacy can disagree with an active generation; and an API source has no artifact-ready state.
- No implementation existed before this review, so there was no runtime migration to repair or roll back. No `web/**` file, dependency, browser data, publisher, deployment, model, engine, worker, tokenizer, or API implementation changed in this design PR.

## What changed

Rebased the design from `26e1473a1` to `27eb6fb84` onto `381ae4d98`, then revised it to `49a39b52e`. Registered separate ready item #3500 rather than reopening completed #3440. Kept Preact + Vite + TypeScript but split delivery into design, foundation, core workflows, parity/migration, and cutover stages.

The foundation is now limited to the typed store, narrow adapters, fixture-backed migration/tests, build configuration, and two additive model-source hooks: structured progress and optional operation metadata. It does not mount/replace the page, invoke the publisher, or touch real browser data. Engine, worker, tokenizer, and API implementations stay unchanged.

Resolved all five builder questions by decision: use additive progress/operation hooks rather than DOM scraping or a duplicate downloader; keep `modelSources.v1` as the live compatibility/provider key without a second secret store; disable new runs on selection/artifact divergence; pin source/privacy per generation; and give API sources a separate selected/test state rather than artifact readiness. Added stale-operation rejection, the full persistence-key inventory, native dialogs/progress, and final parity/cutover gates.

Size before this report: five files, 261 insertions and two deletions. This report is the only additional file. D76 scope advisories cover the explicitly reviewed new item, design, and canonical issue-map paths; the requested rebase also puts the pre-existing design commit before this ORDER, so commit-order remains advisory only. No size cap applies.

## Measurements

- `git diff --exit-code 7a1cbcffc..381ae4d98 -- web/studio`: empty; current source is exactly what the original design reviewed.
- Current source: 11,068 lines across HTML, CSS, and JavaScript, with no component build system.
- `node web/studio/verify_studio_page.mjs --strict-router`: **PASS**, both strict pages clean, no failures. This is the existing static check, not a new browser/worker/accessibility result.
- `roadmap check`: 126 items OK, with #3500 ready and both dependencies done. Roadmap render is current.

## Checks run at the head

The source and static-page checks above ran at `49a39b52e`. After this final report commit, run the full 11 gates, claim wording, roadmap/render/diff checks, and local policy. Post exact-head results on the PR before fresh review and queueing. No npm install, production Vite build, browser run, model load, publisher run, or real-data migration was performed for this design-only unit.

## Skills used

Rebase/isolated-worktree workflow, scope guard, solution integrity, brainstorming and independent builder review, systematic source verification, and modern web guidance for native dialogs, semantic controls, accessible names, live regions, and progress. No model compute or paid resource was used.

## Still open

#3500 remains ready for its separate foundation ORDER at the merged item base. No implementation or cutover is approved by this design alone; production publication remains a separate owner decision. Unrelated sync warnings for #3501–#3507 were left untouched.

## Plain summary

The upgrade is no longer a vague big-bang rewrite. It is a phased migration with a small testable foundation, explicit state/privacy/persistence ownership, and no live-page or engine changes until later parity gates pass.
