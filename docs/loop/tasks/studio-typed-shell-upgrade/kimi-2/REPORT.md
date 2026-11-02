# REPORT: migrate Studio core workflows to typed components, kimi-2

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Implementation head: `bb6404bc8f0d6bf0f9b94daacf46b94a627f061e`; tree: `817855712f4c5e0fd2deda28b40353de23385b35`. Base: `45540f256394599ee9e2b38d3c24769afd45cae7`. This report follows. Exact delivery-head evidence is posted on the PR.

## What did not work

- Initial component tests needed Preact `act()` flushing, complete typed doubles, and queue-aware deferred saves. A real editor-hook test showed that tab content and the legacy textarea could diverge; they are now synchronized.
- **The first fresh review returned CHANGES.** Save results could affect another file, palette save bypassed the store, discrete prediction used chat, provider/session state became stale, generation completion lacked identity, and endpoint errors were hidden. Shared path-scoped saves, discrete prediction routing, provider/session synchronization, generation IDs/error text, and fail-closed readiness were added.
- **The second fresh review returned CHANGES.** Same-path reopen confused saves, non-HTTP endpoints passed, direct-text hooks required legacy DOM, stop used the current mode, attachments were unused, session returns were optimistic, and editor clearing left stale state. Save IDs, HTTP(S) validation, headless direct-text hooks, pinned stop channels, API-only attachment inclusion, real return values, and data-clear synchronization were added.
- **The third fresh review returned CHANGES.** Prediction completion read a mutable generation ID, so a diagnostic result could terminate a typed prediction. Actual page-hook regressions now execute `runPrediction`, `renderDiscretePrediction`, and `onWorkerMessage` without DOM: a diagnostic result is ignored, and only the matching request completes or errors. `predictionRequests[requestId]` maps each worker request to its generation, with an in-flight guard.
- Stop enablement still followed the current picker. `GenerationSnapshot.canStop` now pins whether the operation is stoppable; API generations remain stoppable after selecting discrete, and discrete prediction remains disabled after selecting API.
- Older same-path failures could still call `onError`, and a dirty tab erased a fresh save error. Save operations now run sequentially, stale callbacks check both path and save ID, and `syncWorkspaceSummary` preserves error state.
- Insert used a fixed filename and did not reach the typed shell. Inserts now use unique names; app-level file/view listeners open a dirty Workspace tab and enable Save. New/select session view events return to Agent. Published models now load and subscribe to the catalog instead of relying on the legacy panel.
- Bulk session clearing exposed partial failure only as a legacy toast. `clearSessions()` now returns `{cleared,total}` and Settings surfaces partial failure through its typed error channel.
- Local policy initially rejected the ORDER's plain-text item; its roadmap link is fixed. Policy passes with disclosed D76 scope, ORDER-correction, and FILEMAP advisories. `roadmap sync --apply` remains blocked by unrelated `cpu-discrete-training-path.md` missing `## What is missing` and a stale ID in `docs/roadmap/issues.json`; I did not edit another lab's item or generated output.
- No real-browser/worker/offline/accessibility/mobile result is claimed. jsdom and actual-module/page-hook fixtures exercise behavior, while strict static checks cover the legacy pages. Real File System Access, runtime focus, real model/API trajectories, and real browser-data migration remain unverified.

## What changed

Implemented AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings, composed by unmounted `StudioApp`. AgentComposer, WorkspaceEditor, ModelPicker, and Inspector subscribe to one lifecycle store; AppShell, SessionList, Guide, and Settings are prop/callback-coordinated. Added native Modal/progress semantics and responsive/high-contrast CSS.

Added typed workflow/generation/save adapters and narrow public hooks for direct-text generation and prediction, editor content, settings/templates/data clearing, and attachments. Existing file, session, provider, model, and API implementations retain their behavior and storage. `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `studio_api.js`, engine/WASM, and model artifacts are unchanged.

Added request-mapped prediction IDs, operation-scoped sequential saves, generation IDs/error details, pinned stop capability, provider/session/catalog synchronization, HTTP(S)-only readiness, and public exports. No cutover, publisher, deployment, real browser-data migration, new runtime dependency, or external asset was introduced.

Diff against `origin/main` at `45540f256`: 50 files, 3,231 insertions and 74 deletions at the implementation head. This report follows. No size cap applies.

## Parity record

| Surface | Implemented and tested here | Still required before cutover |
|---|---|---|
| Shell and palette | Four destinations, one status region, keyboard palette, path-and-ID-scoped shared save | Real keyboard/focus return, responsive/mobile and accessibility checks |
| Models | Catalog load/subscription, load forwarding, native progress, capability metadata, selected provider test/edit/remove, no key in removal state | Real downloads, hash/tokenizer/worker readiness and trajectory checks |
| Workspace | File/tab events, sequential path-and-ID-scoped saves, actual legacy textarea/content synchronization, insert/editor clear synchronization | Real File System Access picker, import/download and disk-save parity |
| Agent | Actual request-correlated page prediction, direct-text send, pinned source/privacy/generation/stop, attachments, six templates, message actions | Real local worker and API streaming/error/stop trajectories |
| Sessions and settings | Guarded session operations, preferences, confirmed clearing with refresh and typed partial failure | Real storage inspection, full legacy command parity and migration |
| Inspector and Guide | On-demand details, current privacy and accurate limits | Visual desktop/mobile/light/dark review |

## Measurements

- **103 Vitest tests pass across 19 files:** 30 lifecycle, 14 model-source adapter, 9 migration, 6 generation, 4 save adapter, 1 legacy bridge, 1 actual legacy editor hook, 2 actual page prediction hooks, 1 main, and 35 component/composition tests.
- **8 Node model-source tests pass.** **111 tests total, 0 failures.** The actual page prediction tests prove diagnostic results cannot complete a typed prediction and matching success/error results use the correct generation ID.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 112.55 kB (gzip 30.97 kB) and `dist/studio-foundation.css` 3.39 kB (gzip 1.11 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. Static evidence only, not new runtime browser evidence.

## Checks run at the head

At implementation head `bb6404bc8`, `npm test`, typecheck, build, Node model-source tests, and legacy strict-page checks pass. Final repository gates, claim wording, local policy, diff/status, and exact delivery commit/tree are rerun after this report and recorded on the PR before queueing. Fresh exact-head review and green GitHub checks remain required.

No test used real browser data, credentials, a real model, network, pod, deployment, or production publish. Modern Web Guidance informed native dialog (`dialog`) and progress (`progress`) behavior; `navigation-drawer` informed the labeled `aria-controls`/`aria-expanded` trigger without another modal/focus system. README distinguishes native dialog semantics from pending real-browser focus verification.

## Skills used

Brainstorming against the approved design, writing/executing plans, isolated worktrees, TDD with failing-first cycles, systematic debugging, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute or paid resource was used.

## Still open

Full real-browser feature parity, worker/offline/mobile/accessibility checks, one-time browser-data migration, and cutover remain. The pre-existing R4G1 assertion and real File System Access manual/headed verification gap remain out of scope. The unrelated roadmap sync failure remains disclosed. #3500 remains ready for parity/migration; production publication is a separate owner decision.

## Plain summary

The Studio implements all eight core components with 111 passing tests, including actual page prediction correlation regressions. Stateful views share lifecycle state while direct-text generation, sequential saving, provider configuration, catalog loading, and data controls continue through the existing browser modules. The live page has not been replaced; this is not a deployment or full-parity claim.
