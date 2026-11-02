# REPORT: implement the Studio typed lifecycle foundation, kimi-1

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Implementation head: `601bd29f0115d549f4af75ddb55410fb332c5c31`; only this report and the matching test-file scope registration follow.

## What did not work

- The first lifecycle implementation nested resident identity differently from the public test API and dropped operation source identity. It was corrected to `{ sourceId, schema }` from the operation.
- TypeScript initially lacked a usable declaration for the JavaScript model-source API, and Vite's base config type did not accept the Vitest section. The declaration now sits beside `model_sources.js`, and config uses `vitest/config`.
- The published-load wrapper initially dropped operation metadata. Unknown download totals also leaked into finalization. Both were fixed; legacy calls without metadata still pass.
- A full `npm test` initially picked up the Node-specific `.mjs` suite and failed because it is not a Vitest suite. Vitest is now restricted to `src/**/*.test.ts`; Node runs the JavaScript hooks separately.
- Local policy correctly refused the required REPORT while it was missing. This report completes it. D76 advisories are limited to the reviewed plan/declaration/test scope registrations and known FILEMAP gaps; no implementation-scope waiver is used.

## What changed

Added a typed lifecycle store with artifact operation IDs, real byte progress, indeterminate finalization, stale-event rejection, run-readiness separation, API selected/test states, file save states, and per-generation source/privacy snapshots.

Added two additive model-source hooks: structured `uor-model-load-progress` events and optional operation metadata for published, URL, local-file, and bundled loads. Stale completion returns `{ stale: true }` without selecting, persisting, or emitting source-changed. Added the narrow adapter that forwards existing methods and only accepts worker readiness for artifact-ready state.

Added the pure migration snapshot reader and deterministic copy-forward plan. `modelSources.v1` remains the only live provider/compatibility key; there is no second secret store, write, marker, or real browser-data access.

Added strict TypeScript, Vite library build, Vitest/jsdom configuration, and pinned dependencies. No component, page, publisher, deployment, engine, worker, tokenizer, or `studio_api.js` change was made. #3500 remains ready for the next workflow unit.

Size before this report and the matching test-file scope line: 18 files, 4,379 insertions and 14 deletions. Final total: 19 files, 4,431 insertions and 14 deletions. No size cap applies.

## Measurements

- **32 Vitest tests pass:** 19 lifecycle, 5 adapter, 7 migration, and 1 public entry.
- **7 Node model-source tests pass:** real progress, unknown totals, stale completion, legacy calls, published metadata, local-file finalization, and bundled finalization.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS, `dist/studio-foundation.js` 43.88 kB, generated output uncommitted.
- `node web/studio/verify_studio_page.mjs --strict-router`: PASS, both strict pages clean with zero failures. No new browser, accessibility, or model-capability claim is made.

## Checks run at the head

- `npm test`: 4 files, 32 tests, 0 failures.
- `node web/studio/assets/js/model_sources.test.mjs`: 7 tests, 0 failures.
- `npm run typecheck` and `npm run build`: PASS.
- `uor-harness loop gates --full`: 11/11 GREEN at `601bd29f0`, ledger `local/kimi/gates/1793578503-75656`.
- Claim wording and `git diff --check`: PASS. Roadmap scope includes the plan, declaration, and Node test file; no harness shared file was added to item scope.
- Local policy initially refused the missing REPORT, as recorded above; it is rerun at the final report head before review and queueing. No npm test used real browser data, a real model, a pod, a deployment, or a production publish.

## Skills used

Writing/executing plans, TDD with failing-first cycles, isolated worktrees, solution integrity, scope guard, systematic debugging, modern web guidance for progress and privacy, and independent builder/delivery review. No model compute or paid resource was used.

## Still open

Core workflow components and full parity/cutover remain separate units. The eventual legacy/verifier retarget must be tested without weakening coverage, including the router demo. The pre-existing R4G1 assertion remains out of scope. The real File System Access picker still requires manual/headed verification. Production publication remains a separate owner decision.

## Plain summary

The foundation now provides a tested way to keep model selection, loaded artifacts, file saves, and privacy consistent without rewriting the current page or changing the engine. It is a library and test foundation only; the live Studio has not been replaced.
