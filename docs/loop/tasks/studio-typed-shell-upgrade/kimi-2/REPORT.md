# REPORT: migrate Studio core workflows to typed components, kimi-2

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Implementation head: `64b54176fe7aaebd70204b24f27b8caab446f78c`, merge-forwarded with unrelated `main` changes through `175fca2a8d9c1f3d9b07e6ddc64fc5a57aaece7f`. This report follows. Exact delivery-head evidence is posted on the PR.

## What did not work

- Several first component assertions ran before Preact flushed effects/state. These were test-harness timing errors, corrected with `act()`, not production rewrites. Mock return types also needed literal annotations as the typed API surface grew.
- The initial mock model-source and UI objects did not mirror the complete public interfaces after provider and attachment methods were added. They were completed instead of weakening the declarations.
- `roadmap sync --apply` is blocked on current `main` by two unrelated errors: `cpu-discrete-training-path.md` lacks `## What is missing`, and `docs/roadmap/issues.json` contains an ID with no matching item. The same failure remained after merging `175fca2a8`. I did not edit another lab's item or hand-edit generated output to bypass it.
- No real-browser/worker/offline/accessibility/mobile result is claimed. The components are exercised in jsdom with synthetic APIs, while the existing strict static page check runs against the legacy pages. Real File System Access, runtime focus, real model/API trajectories, and real browser-data migration remain unverified here.

## What changed

Added change-only lifecycle subscriptions and `useLifecycle`, a native Modal, AppShell, CommandPalette, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, Settings, and the unmounted `StudioApp` composition. All eight approved core components now consume one store and expose tested enabled/disabled states.

Added typed workflow/generation adapters and narrow additive public hooks for editor content, generation send/stop/completion, settings updates/templates/data clearing, and single-attachment removal. Existing file, session, provider, model, and API implementations remain the owners of their behavior and storage. `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `studio_api.js`, the engine/WASM boundary, and model artifacts are unchanged.

Added `canPredict` capability forwarding, truthful native progress, next-token-versus-chat labels, responsive/high-contrast `src/studio.css`, and public library exports. No entry-point cutover, publisher, deployment, real browser-data migration, new runtime dependency, or external asset was introduced.

Diff against `origin/main` at `175fca2a8` before this report: 43 files, 2,435 insertions and 6 deletions. The README, roadmap item, and REPORT follow. No size cap applies.

## Parity record

| Surface | Implemented and tested here | Still required before cutover |
|---|---|---|
| Shell and palette | Four destinations, one main/status region, drawer state, keyboard filtering/execution, disabled commands | Real keyboard/focus return, responsive/mobile and accessibility checks |
| Models | Load forwarding, native byte progress, stale/error/readiness states, capability metadata, provider add/use/test/edit/remove without rendering an existing key | Real downloads, hash/tokenizer/worker readiness and trajectory checks |
| Workspace | Folder/file/tab event synchronization, dirty/save/error states, save-as/close through existing API | Real File System Access picker, import/download and disk-save parity |
| Agent | Pinned source/privacy, prediction labels, attachments, six templates, send/stop, message actions | Real local worker and API streaming/error/stop trajectories |
| Sessions and settings | New/select/rename/duplicate/export/delete, appearance/editor preferences, confirmed clearing | Real storage inspection, full legacy command parity and migration |
| Inspector and Guide | On-demand details and accurate current limits | Visual desktop/mobile/light/dark review |

## Measurements

- **79 Vitest tests pass across 16 files:** 28 lifecycle, 14 model-source adapter, 9 migration, 3 generation, 1 legacy bridge, 1 main, and 23 component/composition tests.
- **8 Node model-source tests pass.** **87 tests total, 0 failures.**
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 107.81 kB (gzip 29.85 kB) and `dist/studio-foundation.css` 3.39 kB (gzip 1.11 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. This is a static check, not new runtime browser evidence.

## Checks run at the head

At implementation head `64b54176f`, `npm test`, typecheck, build, Node model-source tests, and legacy strict-page checks pass. Final repository gates, claim wording, local policy, diff/status, and exact delivery commit/tree are rerun after this report and recorded on the PR before queueing. Fresh exact-head review and green GitHub checks remain required.

No test used real browser data, credentials, a real model, network, pod, deployment, or production publish. Modern Web Guidance informed native dialog (`dialog`) and progress (`progress`) behavior; the navigation drawer guidance (`navigation-drawer`) informed the labeled `aria-controls`/`aria-expanded` trigger without adding a second modal/focus system.

## Skills used

Brainstorming against the existing approved design, writing/executing plans, isolated worktrees, TDD with failing-first cycles, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute or paid resource was used.

## Still open

Full real-browser feature parity, worker/offline/mobile/accessibility checks, one-time browser-data migration, and cutover remain. The pre-existing R4G1 assertion and real File System Access manual/headed verification gap remain out of scope. The unrelated roadmap sync failure above remains unresolved and must not be concealed. #3500 remains ready for the parity/migration unit; production publication is a separate owner decision.

## Plain summary

The Studio now has a complete typed core-workflow library with 87 passing tests. It keeps model, file, session, and privacy state coherent while calling the existing browser modules. The live page has not been replaced, and this is not a deployment or full-parity claim.
