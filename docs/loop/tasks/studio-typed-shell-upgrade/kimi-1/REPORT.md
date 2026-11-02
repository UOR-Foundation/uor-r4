# REPORT: implement the Studio typed lifecycle foundation, kimi-1

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Draft PR: #3520. Verified implementation head: `927fbd41a676dfa8551671f99acda952479d5e2c`; tree: `ec1ae33bf28baa5caeaff28eaf8e5609f7136509`. Only this report follows that implementation. Exact delivery-head evidence is posted on the PR.

## What did not work

- The first lifecycle implementation nested resident identity differently from the public test API and dropped operation source identity. The JavaScript declaration and Vite/Vitest typing also needed correction. The published-load wrapper dropped operation metadata, unknown totals leaked into finalization, and Vitest initially picked up the Node-specific `.mjs` suite. These implementation defects were corrected before the first review.
- **The fresh review at `d03e6b533` returned CHANGES.** Worker results could be attributed to a newer load, API test completion could relabel a subsequently selected source, and omitted IDs were rejected. Raw `theme` values were parsed as JSON; malformed entries could receive destinations; the live provider plan retained secret-bearing values. Generation privacy could use a caller-supplied endpoint; hostname prefixes could misclassify public DNS names. The real source result omitted `expectedSha256`, resident metadata was reduced to schema, rejected loads had no error transition, and folder-open/file-close/session-activate states were unreachable. Tests and fixes cover each finding.
- **The second fresh review at `a6a1df522` returned CHANGES.** The worker queue awaited only `postMessage`, not terminal completion, and an old API test could mark a changed endpoint passed under the same source ID. That head was not queued. The queue now awaits terminal success/error before releasing the next load; same-name/same-hash replacement and late-success regressions cover it. API completion additionally requires the originally tested endpoint to remain selected.
- **The third fresh review at `39818db33` returned CHANGES.** It confirmed those two fixes but found that explicit API selection did not invalidate an in-flight metadata load in `model_sources.js`, allowing a late completion to reselect WASM. It also found that a runtime API selection without an endpoint could be labeled `localWASM`. Neither head was queued.
- Metadata-bearing loads now capture a selection epoch. Explicit `setActive` increments it; late loads return stale without selecting, persisting, or emitting a WASM source change. Legacy calls without metadata retain their behavior. The deferred-fetch regression fails before the fix and passes afterward. Generation now rejects a missing API endpoint into session error instead of classifying it as local; its regression also fails first and passes afterward.
- Theme remains a raw string; malformed entries are non-copyable; `modelSources.v1` has `raw: null`, `value: null`, and `destinationKey: null` in the plan. Generation snapshots the selected source, invalid endpoints fail closed, exact IP-literal ranges cover `fe80::/10`, and DNS names default to remote. The hash reaches the worker, resident verification/capability metadata is retained, rejected loads enter `artifact/failed`, unrelated errors do not revoke readiness, and the missing file/session transitions exist.
- The roadmap no longer says no implementation exists and remains ready for later workflows. Local policy correctly refused the required REPORT while it was missing and later caught two headings dropped in a rewrite; both are restored. Expected D76 advisories are limited to reviewed plan/declaration/test scope registrations and known FILEMAP gaps; no implementation-scope waiver is used.

## What changed

Added a typed lifecycle store with artifact operation IDs, real byte progress, indeterminate finalization, stale-event rejection, run-readiness separation, source/endpoint-scoped API tests, file/session states, and per-generation source/privacy snapshots.

Added structured `uor-model-load-progress` events and optional operation metadata for published, URL, local-file, and bundled loads. Stale completion returns `{ stale: true }` without selecting, persisting, or emitting source-changed. Explicit source selection supersedes in-flight metadata loads. The narrow adapter uses existing model-source methods and completion-serialized worker readiness rather than reimplementing the engine.

Added the pure migration snapshot reader and deterministic copy-forward plan. `modelSources.v1` remains the only live provider/compatibility key; there is no second secret store, write, marker, or real browser-data access.

Added strict TypeScript, Vite library build, Vitest/jsdom configuration, and pinned dependencies. No component, page, publisher, deployment, engine, worker, tokenizer, or `studio_api.js` change was made. #3500 remains ready for the next workflow unit.

Diff against `origin/main` at `ee534711f78185f1f726706493dbe5d479a8937d`: 19 files, 4,820 insertions and 16 deletions at the verified implementation head. The report-only update is included in the final PR size. No size cap applies.

## Measurements

- **51 Vitest tests pass:** 27 lifecycle, 14 adapter, 9 migration, and 1 public entry.
- **8 Node model-source tests pass:** real progress, unknown totals, stale completion, explicit-selection supersession, legacy calls, published metadata, local-file finalization, and bundled finalization.
- **59 tests total, 0 failures.** The omitted-ID, same-artifact worker replacement, same-ID endpoint-change, explicit-selection-during-load, and missing-endpoint regressions each failed for their intended reason before their fixes.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS, `dist/studio-foundation.js` 47.41 kB (gzip 13.45 kB), generated output uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both strict pages clean with zero failures. No new browser, accessibility, or model-capability claim is made.

## Checks run at the head

- At `927fbd41a676dfa8551671f99acda952479d5e2c`, tree `ec1ae33bf28baa5caeaff28eaf8e5609f7136509`, the complete foundation suite, typecheck, build, Node hooks, and static strict-page check pass after merging `origin/main` at `ee534711`.
- `uor-harness loop gates --full`: 11/11 GREEN at that implementation head, ledger `local/kimi/gates/1793583846-57071`.
- `uor-harness loop pr-check --local`: PASS at that implementation head, with only the disclosed D76 scope and FILEMAP advisories.
- Claim wording and `git diff --check`: PASS; worktree clean at the checked implementation head.
- The final report-only commit, exact delivery head/tree, fresh review, and GitHub checks are recorded on the PR before queueing.

No npm test used real browser data, a real model, a pod, a deployment, or a production publish. Modern Web Guidance informed native progress semantics, no invented percentages/stages, and fail-closed endpoint privacy; no component UI is included here.

## Skills used

Writing/executing plans, TDD with failing-first cycles, isolated worktrees, solution integrity, scope guard, systematic debugging, modern web guidance for progress and privacy, verification before completion, and independent builder/delivery review. No model compute or paid resource was used.

## Deferred minors

The reviews' minor items remain explicit rather than expanding this unit: adapter listener teardown; defensive/read-only `getState()`; avoiding `loaded: 0` for indeterminate finalization without prior progress; per-caller rather than module-global operation counters; per-key migration defaults beyond `null`; store-level API test endpoint/attempt identity in addition to adapter enforcement; and serialization or attempt tokens for concurrent retests of the same endpoint. Real API completion remains adapter-owned in this foundation; no components are introduced. These do not establish a live-page or cutover claim.

## Still open

Core workflow components and full parity/cutover remain separate units. The eventual legacy/verifier retarget must be tested without weakening coverage, including the router demo. The pre-existing R4G1 assertion remains out of scope. The real File System Access picker still requires manual/headed verification. Production publication remains a separate owner decision.

## Plain summary

The foundation provides a tested way to keep model selection, loaded artifacts, file saves, and privacy consistent without rewriting the current page or changing the engine. It is a library and test foundation only; the live Studio has not been replaced.
