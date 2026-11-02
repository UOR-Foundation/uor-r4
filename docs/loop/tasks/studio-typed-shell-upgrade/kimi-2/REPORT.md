# REPORT: migrate Studio core workflows to typed components, kimi-2

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Implementation head: `cd76f748e4043547c07cbb092567c458562ef6a3`; tree: `2c7319b4dbd50dbbc3810ea88f8c0c816d819bac`. Base: `43809a41aa96ab64a4aed4328bba7487b14d8e42`. This report follows. Exact delivery-head evidence is posted on the PR.

## What did not work

- Initial component tests needed Preact `act()` flushing, complete typed doubles, and queue-aware deferred saves. A real editor-hook test showed that tab content and the legacy textarea could diverge; they are now synchronized.
- **The first fresh review returned CHANGES.** Save results could affect another file, palette save bypassed the store, discrete prediction used chat, provider/session state became stale, generation completion lacked identity, and endpoint errors were hidden. Shared path-scoped saves, discrete prediction routing, provider/session synchronization, generation IDs/error text, and fail-closed readiness were added.
- **The second fresh review returned CHANGES.** Same-path reopen confused saves, non-HTTP endpoints passed, direct-text hooks required legacy DOM, stop used the current mode, attachments were unused, session returns were optimistic, and editor clearing left stale state. Save IDs, HTTP(S) validation, headless direct-text hooks, pinned stop channels, API-only attachment inclusion, real return values, and data-clear synchronization were added.
- **The third fresh review returned CHANGES.** Prediction completion read a mutable generation ID, so diagnostics could terminate a typed prediction. Actual page-hook regressions now execute the real functions without DOM: diagnostics render without completing the typed generation, and only matching requests complete or error. `predictionRequests[requestId]` maps worker requests to generation IDs, with in-flight and chat-generation guards.
- **The fourth fresh review returned CHANGES.** Prediction could overwrite chat generation IDs, diagnostics stopped rendering, timestamp-only inserts collided, and the top-level alert survived successful retries. Prediction is refused during active chat, diagnostics render without completion, insert names add a monotonic sequence, and successful actions clear the alert.
- **The fifth fresh review returned CHANGES.** Queued saves could write the wrong active tab, chat could start during prediction, incompatible artifacts enabled Send, active mode labels followed the picker, clear-all created a replacement session, and Inspector attachments were stale. Queued writes now abort before touching a mismatched path/save ID, chat is excluded during prediction, capability-gated readiness and mode labels are pinned, clear-all suppresses replacement sessions, and Inspector subscribes to attachment changes.
- Stop enablement is pinned by `GenerationSnapshot.canStop`. Saves are serialized and checked by path/save ID, with stale callback suppression and dirty failed-save preservation. Published models load and subscribe to the catalog. Session clearing returns `{cleared,total,error?}` so Settings reports partial/unavailable results.
- Local policy initially rejected the ORDER's plain-text item; its roadmap link is fixed. Policy passes with disclosed D76 scope, ORDER-correction, and FILEMAP advisories. `roadmap sync --apply` remains blocked by unrelated `cpu-discrete-training-path.md` missing `## What is missing` and a stale ID in `docs/roadmap/issues.json`; I did not edit another lab's item or generated output.
- No real-browser/worker/offline/accessibility/mobile result is claimed. jsdom and actual-module/page-hook fixtures exercise behavior, while strict static checks cover the legacy pages. Real File System Access, runtime focus, real model/API trajectories, and real browser-data migration remain unverified.

## What changed

Implemented AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings, composed by unmounted `StudioApp`. AgentComposer, WorkspaceEditor, ModelPicker, and Inspector subscribe to one lifecycle store; AppShell, SessionList, Guide, and Settings are prop/callback-coordinated. Added native Modal/progress semantics and responsive/high-contrast CSS.

Added typed workflow/generation/save adapters and narrow public hooks for direct-text generation and prediction, editor content, settings/templates/data clearing, and attachments. Existing file, session, provider, model, and API implementations retain their behavior and storage. `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `studio_api.js`, engine/WASM, and model artifacts are unchanged.

Added request-mapped prediction IDs, operation-scoped sequential saves, generation IDs/error details, pinned stop capability, provider/session/catalog synchronization, HTTP(S)-only readiness, unique insert names, and public exports. No cutover, publisher, deployment, real browser-data migration, new runtime dependency, or external asset was introduced.

Diff against `origin/main` at `43809a41`: 50 files, 3,385 insertions and 77 deletions at the implementation head. This report follows. No size cap applies.

## Parity record

| Surface | Implemented and tested here | Still required before cutover |
|---|---|---|
| Shell and palette | Four destinations, one status region, keyboard palette, path-and-ID-scoped shared save, successful-retry alert clearing | Real keyboard/focus return, responsive/mobile and accessibility checks |
| Models | Catalog load/subscription, load forwarding, native progress, capability metadata, selected provider test/edit/remove, no key in removal state | Real downloads, hash/tokenizer/worker readiness and trajectory checks |
| Workspace | File/tab events, sequential path-and-ID-scoped saves with pre-write guards, actual legacy textarea/content synchronization, insert/editor clear synchronization | Real File System Access picker, import/download and disk-save parity |
| Agent | Actual request-correlated page prediction, direct-text send with prediction/chat exclusion, pinned source/privacy/generation/stop, attachments, six templates, message actions | Real local worker and API streaming/error/stop trajectories |
| Sessions and settings | Guarded session operations, preferences, confirmed clearing with refresh and typed partial/unavailable results | Real storage inspection, full legacy command parity and migration |
| Inspector and Guide | On-demand details with attachment subscription, current privacy and accurate limits | Visual desktop/mobile/light/dark review |

## Measurements

- **111 Vitest tests pass across 19 files:** 30 lifecycle, 14 model-source adapter, 9 migration, 6 generation, 5 save adapter, 1 legacy bridge, 1 actual legacy editor hook, 5 actual page prediction hooks, 1 main, and 39 component/composition tests.
- **8 Node model-source tests pass.** **119 tests total, 0 failures.** Actual page tests prove prediction/chat exclusion, diagnostic rendering without completion, correlated success/error, and no wrong-tab queued write.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 113.53 kB (gzip 31.20 kB) and `dist/studio-foundation.css` 3.39 kB (gzip 1.11 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. Static evidence only, not new runtime browser evidence.

## Checks run at the head

At implementation head `cd76f748e`, `npm test`, typecheck, build, Node model-source tests, and legacy strict-page checks pass. Final repository gates, claim wording, local policy, diff/status, and exact delivery commit/tree are rerun after this report and recorded on the PR before queueing. Fresh exact-head review and green GitHub checks remain required.

No test used real browser data, credentials, a real model, network, pod, deployment, or production publish. Modern Web Guidance informed native dialog (`dialog`) and progress (`progress`) behavior; `navigation-drawer` informed the labeled `aria-controls`/`aria-expanded` trigger without another modal/focus system. README distinguishes native dialog semantics from pending real-browser focus verification.

## Skills used

Brainstorming against the approved design, writing/executing plans, isolated worktrees, TDD with failing-first cycles, systematic debugging, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute or paid resource was used.

## Still open

Full real-browser feature parity, worker/offline/mobile/accessibility checks, one-time browser-data migration, and cutover remain. The pre-existing R4G1 assertion and real File System Access manual/headed verification gap remain out of scope. The unrelated roadmap sync failure remains disclosed. #3500 remains ready for parity/migration; production publication is a separate owner decision.

## Plain summary

The Studio implements all eight core components with 119 passing tests, including actual page prediction correlation, chat exclusion, queued-save binding, capability readiness, insert uniqueness, and attachment refresh. Stateful views share lifecycle state while direct-text generation, sequential saving, provider configuration, catalog loading, and data controls continue through existing browser modules. The live page has not been replaced; this is not a deployment or full-parity claim.
