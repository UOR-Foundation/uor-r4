# REPORT: implement the Studio typed lifecycle foundation, kimi-1

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Draft PR: #3520. Final implementation head: `6aea969f0dd3ea325fb5a1b5cca5a3e1f0877053`; tree: `cb1af77f82cde1ea86bdb8977a8142fe32471b5c`. This report and the synchronized roadmap input follow. Exact delivery-head evidence is posted on the PR.

## What did not work

- The first lifecycle implementation nested resident identity differently from the public test API and dropped operation source identity. The JavaScript declaration and Vite/Vitest typing also needed correction. The published-load wrapper dropped operation metadata, unknown totals leaked into finalization, and Vitest initially picked up the Node-specific `.mjs` suite. These implementation defects were corrected before the first review.
- **The fresh review at `d03e6b533` returned CHANGES.** Worker results could be attributed to a newer load. Loads are now serialized, `load_model` carries the operation ID, readiness matches the posted operation plus filename/hash when reported, and matching operation errors fail only their own load. Rejected loads enter `artifact/failed`; unrelated errors do not revoke readiness.
- API test completion could relabel a subsequently selected source, and omitted IDs were rejected. Selection now calls the existing `setActive`; test actions and completion are source-ID scoped, and an omitted ID uses the selected source.
- Raw `theme` values were parsed as JSON, malformed entries could receive copy destinations, and the live provider key retained secret-bearing plan values. Theme now remains a raw string; malformed entries are non-copyable; `modelSources.v1` has `raw: null`, `value: null`, and `destinationKey: null` in the plan.
- Generation privacy could use a caller-supplied endpoint, and hostname prefixes could misclassify public DNS names. Generation snapshots the selected source, invalid endpoints fail closed, exact IP-literal ranges cover `fe80::/10`, and DNS names default to remote.
- The real source result omitted `expectedSha256`, resident metadata was reduced to schema, rejected loads had no error transition, and folder-open/file-close/session-activate states were unreachable. The hash now reaches the worker, resident verification/capability metadata is retained, and those transitions exist.
- The first report cited a pre-merge-forward head and the roadmap still said no implementation existed. This record uses the final implementation head and current measurements; #3500 remains ready for later workflows.
- Local policy correctly refused the required REPORT while it was missing. The initial D76 advisories were limited to reviewed plan/declaration/test scope registrations and known FILEMAP gaps; no implementation-scope waiver is used.

## What changed

Added a typed lifecycle store with artifact operation IDs, real byte progress, indeterminate finalization, stale-event rejection, run-readiness separation, source-scoped API tests, file/session states, and per-generation source/privacy snapshots.

Added structured `uor-model-load-progress` events and optional operation metadata for published, URL, local-file, and bundled loads. Stale completion returns `{ stale: true }` without selecting, persisting, or emitting source-changed. The narrow adapter uses existing model-source methods and worker readiness rather than reimplementing the engine.

Added the pure migration snapshot reader and deterministic copy-forward plan. `modelSources.v1` remains the only live provider/compatibility key; there is no second secret store, write, marker, or real browser-data access.

Added strict TypeScript, Vite library build, Vitest/jsdom configuration, and pinned dependencies. No component, page, publisher, deployment, engine, worker, tokenizer, or `studio_api.js` change was made. #3500 remains ready for the next workflow unit.

Diff against `origin/main` at `728cf3f6ce0101e3f7d1cc7e2ebd16d7ab9f021b`: 19 files, 4,740 insertions and 16 deletions. No size cap applies.

## Measurements

- **48 Vitest tests pass:** 26 lifecycle, 12 adapter, 9 migration, and 1 public entry.
- **7 Node model-source tests pass:** real progress, unknown totals, stale completion, legacy calls, published metadata, local-file finalization, and bundled finalization.
- **55 tests total, 0 failures.** The omitted-ID adapter test failed first with `Cannot test a non-selected API source`, then passed after deriving the selected ID.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS, `dist/studio-foundation.js` 47.08 kB (gzip 13.32 kB), generated output uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both strict pages clean with zero failures. No new browser, accessibility, or model-capability claim is made.

## Checks run at the head

- At `38a32c5076d6fa3b2a283962d0cb34bd365d0393`, the complete foundation suite, typecheck, build, Node hooks, and static strict-page check pass after merging `origin/main` at `728cf3f6`.
- `uor-harness loop gates --full`: 11/11 GREEN, ledger `local/kimi/gates/1793581218-34127`.
- Claim wording and `git diff --check`: PASS; worktree clean at the checked head.
- Local policy initially failed because this rewritten report omitted its required `Checks run at the head` and `Skills used` headings. Those headings are restored and policy is rerun before queueing. Expected D76 advisories are limited to the reviewed plan/declaration/test scope registrations and known FILEMAP gaps; no implementation-scope waiver is used.
- The final report-only commit, exact delivery head/tree, fresh review, and GitHub checks are recorded on the PR before queueing.

No npm test used real browser data, a real model, a pod, a deployment, or a production publish. Modern Web Guidance informed native progress semantics, no invented percentages/stages, and fail-closed endpoint privacy; no component UI is included here.

## Skills used

Writing/executing plans, TDD with failing-first cycles, isolated worktrees, solution integrity, scope guard, systematic debugging, modern web guidance for progress and privacy, verification before completion, and independent builder/delivery review. No model compute or paid resource was used.

## Deferred minors

The review's minor items remain explicit rather than expanding this unit: adapter listener teardown, defensive/read-only `getState()`, avoiding `loaded: 0` for an indeterminate finalization without prior progress, per-caller rather than module-global operation counters, and per-key migration defaults beyond `null`. These do not establish a live-page or cutover claim.

## Still open

Core workflow components and full parity/cutover remain separate units. The eventual legacy/verifier retarget must be tested without weakening coverage, including the router demo. The pre-existing R4G1 assertion remains out of scope. The real File System Access picker still requires manual/headed verification. Production publication remains a separate owner decision.

## Plain summary

The foundation provides a tested way to keep model selection, loaded artifacts, file saves, and privacy consistent without rewriting the current page or changing the engine. It is a library and test foundation only; the live Studio has not been replaced.
