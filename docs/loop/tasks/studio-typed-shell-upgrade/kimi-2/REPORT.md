# REPORT: migrate Studio core workflows to typed components, kimi-2

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Implementation head: `7d50190df50c5275b3ebc0cc4046c054f09628c0`; tree: `fef5b08a9f61ce83c80854807caaca0b9ea10b40`. Base: `af1979886ebb29b98254ef6b0d63aa753a879d5f`. This report follows. Exact delivery-head evidence is posted on the PR.

## What did not work

- Initial component assertions needed Preact `act()` flushing and complete literal-typed API doubles. A real editor-hook test showed that tab content and the legacy textarea could diverge; they are now synchronized.
- The composed status could label a missing endpoint local or use an old generation's privacy. Current active-generation or current-selection privacy is used, with an explicit last-generation label only when no source is selected.
- **The first fresh review returned CHANGES.** Save results could affect another file, palette save bypassed the store, discrete prediction used chat, provider and session state became stale, generation completion lacked identity, and endpoint errors were hidden. Path-scoped shared saves, discrete prediction routing, provider/session synchronization, generation IDs/error text, and fail-closed readiness were added with regressions.
- **The second fresh review returned CHANGES.** A same-path reopen could confuse save operations, so save IDs are now required alongside paths. Non-HTTP endpoints passed classification; HTTP(S) is now required. Artifact selection resets stale API test state.
- Direct-text send/predict still depended on legacy input/result DOM. Those hooks now accept text directly, optional chat/predictor nodes are guarded, and prediction completion fires even without result nodes. Matching prediction worker errors terminate with typed error details.
- Stop used the currently selected mode instead of the active generation. It now uses the generation's pinned local/API channel. Prediction Stop is disabled because the synchronous prediction operation has no cancellation path.
- Stored attachments were never included in API requests. Existing workspace context is now prepended as a user context message only on the API path; local generation is unchanged.
- New/select session wrappers returned success during legacy refusal. Their real boolean results now propagate. Selected-provider removal also restores legacy WASM selection, and clearing editor tabs closes lifecycle file state even when Workspace is unmounted.
- Local policy initially rejected the ORDER's plain-text item; its roadmap link is fixed. Policy passes with disclosed D76 scope, ORDER-correction, and FILEMAP advisories. `roadmap sync --apply` remains blocked by unrelated `cpu-discrete-training-path.md` missing `## What is missing` and a stale ID in `docs/roadmap/issues.json`; I did not edit another lab's item or generated output.
- No real-browser/worker/offline/accessibility/mobile result is claimed. jsdom and actual-module fixtures exercise behavior, while strict static checks cover the legacy pages. Real File System Access, runtime focus, real model/API trajectories, and real browser-data migration remain unverified.

## What changed

Implemented the eight approved components: AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings, composed by unmounted `StudioApp`. Stateful views subscribe to one lifecycle store; Guide, Settings, and shell controls are coordinated through props and callbacks. Added native Modal/progress semantics and responsive/high-contrast CSS.

Added typed workflow/generation/save adapters and narrow public hooks for direct-text generation and prediction, editor content, settings/templates/data clearing, and attachments. Existing file, session, provider, model, and API implementations retain their behavior and storage. `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `studio_api.js`, engine/WASM, and model artifacts are unchanged.

Added `canPredict` forwarding, path-and-operation-scoped saves, generation IDs/error details, pinned stop channels, provider/session synchronization, HTTP(S)-only readiness, and public exports. No cutover, publisher, deployment, real browser-data migration, new runtime dependency, or external asset was introduced.

Diff against `origin/main` at `af1979886`: 48 files, 2,988 insertions and 74 deletions at the implementation head. This report follows. No size cap applies.

## Parity record

| Surface | Implemented and tested here | Still required before cutover |
|---|---|---|
| Shell and palette | Four destinations, one status region, keyboard palette, path-and-ID-scoped shared save | Real keyboard/focus return, responsive/mobile and accessibility checks |
| Models | Load forwarding, native progress, capability metadata, selected provider test/edit/remove, no key in removal state | Real downloads, hash/tokenizer/worker readiness and trajectory checks |
| Workspace | File/tab events, path-and-ID-scoped dirty/save/error states, actual legacy textarea/content synchronization, editor clear synchronization | Real File System Access picker, import/download and disk-save parity |
| Agent | Direct-text send/predict hooks, pinned source/privacy/generation IDs and stop channel, discrete prediction, attachments, six templates, send/stop/error retry, message actions | Real local worker and API streaming/error/stop trajectories |
| Sessions and settings | Guarded session operations, preferences, confirmed clearing with refresh and honest partial failure | Real storage inspection, full legacy command parity and migration |
| Inspector and Guide | On-demand details, current privacy and accurate limits | Visual desktop/mobile/light/dark review |

## Measurements

- **97 Vitest tests pass across 18 files:** 30 lifecycle, 14 model-source adapter, 9 migration, 6 generation, 3 save adapter, 1 legacy bridge, 1 actual legacy editor hook, 1 main, and 32 component/composition tests.
- **8 Node model-source tests pass.** **105 tests total, 0 failures.** New regressions cover both reviews' named behaviors, including same-path saves, direct-text prediction, duplicate terminal IDs, non-HTTP endpoints, provider selection/removal, and data clearing.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 111.01 kB (gzip 30.55 kB) and `dist/studio-foundation.css` 3.39 kB (gzip 1.11 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. Static evidence only, not new runtime browser evidence.

## Checks run at the head

At implementation head `7d50190df`, `npm test`, typecheck, build, Node model-source tests, and legacy strict-page checks pass. Final repository gates, claim wording, local policy, diff/status, and exact delivery commit/tree are rerun after this report and recorded on the PR before queueing. Fresh exact-head review and green GitHub checks remain required.

No test used real browser data, credentials, a real model, network, pod, deployment, or production publish. Modern Web Guidance informed native dialog (`dialog`) and progress (`progress`) behavior; `navigation-drawer` informed the labeled `aria-controls`/`aria-expanded` trigger without another modal/focus system. README distinguishes native dialog semantics from pending real-browser focus verification.

## Skills used

Brainstorming against the approved design, writing/executing plans, isolated worktrees, TDD with failing-first cycles, systematic debugging, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute or paid resource was used.

## Still open

Full real-browser feature parity, worker/offline/mobile/accessibility checks, one-time browser-data migration, and cutover remain. The pre-existing R4G1 assertion and real File System Access manual/headed verification gap remain out of scope. The unrelated roadmap sync failure remains disclosed. #3500 remains ready for parity/migration; production publication is a separate owner decision.

## Plain summary

The Studio now implements all eight core components with 105 passing tests, including regressions for both independent reviews. Stateful views share lifecycle state while direct-text generation, file saving, provider configuration, and data controls continue through the existing browser modules. The live page has not been replaced; this is not a deployment or full-parity claim.
