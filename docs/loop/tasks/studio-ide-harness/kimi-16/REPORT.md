# REPORT: make the neutral Studio the default web UI, kimi-16

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `402b4091eb7f7e7ea174f96ed0bbd06cf58fa78b`, tree `5af32980ffc2f95cc52de8ead8e80493184f1f73`. Implementation head: `a4cf1dfe9ee4dc646e8c6a6e3d2f979d07d7a5e9`, tree `d5e17520ecd8d5e062156e0b27fbc384dfa44669`. PR #3903 is draft. Plan: [2026-11-09-studio-default-cutover](../../../superpowers/plans/2026-11-09-studio-default-cutover.md).

## What did not work

- Plain `/studio/` opened the legacy conversation UI. The neutral workbench mounted only for explicit `typed=1`; treating that preview as a completed default cutover was incorrect.
- The first redirect made the source page request `src/main.ts`, violating the source/runtime boundary. A built-entry-only guard then exposed a bare `get` receiver error. `runtime-page-load-red-source.log` and `runtime-page-load-red-receiver.log` preserve the exact tool records.
- The built bootstrap initially left migration inactive. Making activation unconditional then changed an already migrated store from `migrated` to `current`, breaking first-mount identity. Activation now occurs only when inactive, preserving `migrated` on first mount and `current` on marked reload.
- Four legacy visual fixtures and the foundation no-mount test used the pre-cutover default route. They now explicitly select `?legacy=1`; this preserves the legacy assertions instead of accepting the new UI as a substitute.
- The new Vite-config test initially omitted `config.plugins?`, which TypeScript correctly rejected as possibly undefined. The optional chain is present and typecheck passes.
- The first browser run lacked generated WASM artifacts; that was setup failure, not the routing regression. The standard runtime preparer generated them before the red test was accepted. Automatic `loop claim` also refused the item's existing shared roadmap-file scope. This ORDER uses an isolated worktree/protected PR without changing the owner's roadmap files.

## What changed

- `studio-default-typed-entry` in `vite.config.ts` replaces only the built `typed_entry.js` guard with a default typed selection, retaining the full `URLSearchParams` receiver and rejecting a missing preview guard. The source entry remains byte-identical and does not import raw TypeScript by default.
- `typedPreviewRequested` now defaults to typed except for explicit `legacy=1` or `typed=0`. `mountTypedStudio` activates migration only when inactive and applies both explicit theme queries and theme hashes. The rejected source-level redirect is removed, so the original URL and hash remain unchanged.
- Legacy visual and no-mount scenarios explicitly request the legacy interface. The bootstrap, migration, build-transform and full-browser regressions cover default mounting, legacy fallback, theme handling, first migration and marked reload.
- No model, engine, worker, tokenizer, API implementation, `lifecycle_persistence.js`, Tauri, native-consent, dependency, `.github/**`, or public terminal service changed. This is not the shared agent runtime or a useful-model result.

## Measurements

- **455/455** typed tests across **52 files** pass; TypeScript check passes. **47/47** Playwright checks pass. **15/15** typed/Tauri runtime contracts pass. Static page and worker checks pass, including bit-identical save/restore and zero remote requests in that check.
- The runtime page-load trap passes with zero runtime errors and no raw-TypeScript request. Migration success, malformed source, destination conflict and one-shot quota cases all pass, including restored sessions/workspace/settings/attachments/theme/UI, exact UOR arithmetic `17 + 25 = 42`, marked reload, and provider-secret isolation.
- Typed staging has **17 files, 3,063,808 bytes**. The direct **7/7** WASM audit is clean. Artifact SHA-256 `a9e73341213880c107c7a7cc91bf644de926be9655b699eef9e7d11a70a2fea7`, 807,378 bytes; staged and dist `index.html` are identical.
- Actual staged desktop and mobile screenshots show the neutral workbench; the legacy route shows the old interface. Evidence: `local/kimi/default-cutover/staged-desktop.png`, `staged-mobile.png`, `stage.log`, `runtime-page-load.log`, and the focused/full-suite logs in that directory.

## Checks run at the head

- The default URL regression failed before implementation with the legacy URL retained. The bootstrap default, build-transform, already-migrated and optional-type tests each failed for their intended reason before their corresponding fix. Full-suite checks above then passed at `a4cf1dfe9ee4dc646e8c6a6e3d2f979d07d7a5e9` / `d5e17520ecd8d5e062156e0b27fbc384dfa44669`.
- `git diff --check`, staged allowlist/catalog validation and direct staged WASM **7/7** pass. Runtime and migration traps pass. No earlier raw-TypeScript, bare-receiver, setup, fixture, or status-check failure is relabeled as a pass.
- `uor-harness loop pr-check --local` and `uor-harness loop gates --full` are rerun after this REPORT is committed, with exact clean HEAD/tree and retained logs. Fresh exact-head review, required GitHub checks, protected queue acceptance and actual merge are still outstanding at this writing.
- Fresh-main publication and hard-reloaded public default-route verification are also outstanding. `kimi-17` will own the live publication/current records and rollback identity after actual deployment; it will not infer them from this local staged evidence.

## Decisions and rejected alternatives

- Rejected a source-level redirect because it made the default source request raw TypeScript. The built entry alone now defaults to the workbench, preserving the source legacy path.
- Kept activation limited to inactive migration state instead of reactivating every mount. This preserves the first-mount `migrated` state and marked-reload `current` state.
- Preserved legacy screenshot/no-mount coverage with an explicit legacy query instead of deleting those tests or accepting typed output as the expected legacy output.
- Used an isolated worktree and explicit ORDER after the item metadata refused automatic claim; no owner scope or roadmap files were changed to work around that refusal.

## Skills used

Systematic debugging, red/green test-driven development, inline plan execution with a durable ledger, scope/solution-integrity review, real browser acceptance, and verification-before-completion. Modern web guidance was searched for application startup/navigation patterns; no directly applicable guide was used for this source/build-specific cutover.

## Still open

Fresh exact-head approval, required checks, actual protected merge, fresh-main typed build/publication, public default and legacy route verification, and `kimi-17` publication records. Shared agent runtime, parallel tasks, project memory, steering/orchestration, Dispatch, connectors, embedded browser/computer use, live R4 API, signed release and useful-model behavior remain unbuilt.

## Plain summary

The old hosted root was still an opt-in preview, not the Cowork-style default. The built app now mounts the neutral workbench by default with correct migration and explicit legacy fallbacks. Local evidence is green; protected merge and actual public deployment are the remaining proof, not facts to infer from this report.
