# REPORT: migrate Studio core workflows to typed components, kimi-2

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Implementation head: `ed666322bf2595114f452fd53d77a89ff1b42745`; tree: `24c90f53a2be9ca2117550f05205cb94d3bdbab5`. Base: `175fca2a8d9c1f3d9b07e6ddc64fc5a57aaece7f`. This report follows. Exact delivery-head evidence is posted on the PR.

## What did not work

- Several first component assertions ran before Preact flushed effects/state; these test-harness timing errors were corrected with `act()`. Mock return types needed literal annotations, and incomplete model/UI doubles were completed rather than weakening declarations.
- A pre-review integration check found that `setActiveContent` updated the tab but not the legacy textarea read by `saveActive()`. An actual-module regression failed first, then passed after synchronizing the textarea.
- The composed status initially labeled an API selection without an endpoint as `localWASM`. A composed regression failed first, then passed after an explicit guard. The composed privacy status also used a previous generation's privacy after a new selection; it now uses active-generation privacy or the current selection.
- **The fresh review at `f0a7aae6e` returned CHANGES.** Save completion/failure could affect a newly selected file, and palette save bypassed the store. Save actions now carry path identity, `saveActiveFile()` shares editor/palette transitions, and old results cannot overwrite another file's state. Regressions cover an old failure and an older success arriving during a newer save.
- Discrete `canGenerate: false, canPredict: true` artifacts were labeled prediction but sent chat generation. The legacy predictor now returns its operation ID, the adapter calls `predict()` rather than `sendPrompt()`, and the existing `predict_discrete` path supplies completion. Missing/malformed endpoints disable `canRun()` and Send; generation errors preserve text and permit valid-source retry.
- Provider Test used an unselected source and failed under the real adapter. Test now selects first. Removal state contains only `{id,label}`, never the existing key. Editing a selected endpoint reselects it; removing the selected provider moves to selected-not-ready artifact state and disables runs.
- Clearing sessions did not refresh the typed list and could report success while generation blocked deletion. Settings now refreshes the composed list/store, legacy deletion returns actual success, and bulk clearing reports partial failure honestly. Failed new/select/delete actions no longer activate a false state.
- Generation completion had no identity and dropped error text. `GenerationSnapshot` and terminal events now carry generation IDs; stale/duplicate completions are ignored. Error detail reaches the typed alert, while source/privacy remain pinned.
- Local policy initially rejected the ORDER's plain-text item; the roadmap link is fixed. Policy passes with disclosed D76 scope, ORDER-correction, and FILEMAP advisories.
- `roadmap sync --apply` remains blocked by unrelated `cpu-discrete-training-path.md` missing `## What is missing` and a stale ID in `docs/roadmap/issues.json`. I did not edit another lab's item or hand-edit generated output.
- No real-browser/worker/offline/accessibility/mobile result is claimed. jsdom and actual-module fixtures exercise behavior, while strict static checks cover the legacy pages. Real File System Access, runtime focus, real model/API trajectories, and real browser-data migration remain unverified.

## What changed

Added change-only lifecycle subscriptions and `useLifecycle`, native Modal, AppShell, CommandPalette, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, Settings, and unmounted `StudioApp`. All eight approved components consume one store and expose tested enabled/disabled states.

Added typed workflow/generation/save adapters and narrow public hooks for editor content, generation and prediction, settings/templates/data clearing, and attachment removal. Existing file, session, provider, model, and API implementations retain their behavior and storage. `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `studio_api.js`, engine/WASM, and model artifacts are unchanged.

Added `canPredict` forwarding, truthful native progress, prediction-versus-chat labels, path-scoped saves, generation IDs/errors, provider/session synchronization, responsive/high-contrast CSS, and public exports. No cutover, publisher, deployment, real browser-data migration, new runtime dependency, or external asset was introduced.

Diff against `origin/main` at `175fca2a8`: 48 files, 2,863 insertions and 49 deletions at the implementation head. This report follows. No size cap applies.

## Parity record

| Surface | Implemented and tested here | Still required before cutover |
|---|---|---|
| Shell and palette | Four destinations, one status region, keyboard palette, path-scoped shared save | Real keyboard/focus return, responsive/mobile and accessibility checks |
| Models | Load forwarding, native progress, capability metadata, selected provider test/edit/remove, no key in removal state | Real downloads, hash/tokenizer/worker readiness and trajectory checks |
| Workspace | File/tab events, path-scoped dirty/save/error states, actual legacy textarea/content synchronization | Real File System Access picker, import/download and disk-save parity |
| Agent | Pinned source/privacy and generation IDs, discrete predictor routing, attachments, six templates, send/stop/error retry, message actions | Real local worker and API streaming/error/stop trajectories |
| Sessions and settings | Guarded session operations, preferences, confirmed clearing with refresh and honest partial failure | Real storage inspection, full legacy command parity and migration |
| Inspector and Guide | On-demand details, current privacy and accurate limits | Visual desktop/mobile/light/dark review |

## Measurements

- **92 Vitest tests pass across 18 files:** 29 lifecycle, 14 model-source adapter, 9 migration, 6 generation, 2 save adapter, 1 legacy bridge, 1 actual legacy editor hook, 1 main, and 29 component/composition tests.
- **8 Node model-source tests pass.** **100 tests total, 0 failures.** New review regressions failed first for save identity, palette transitions, discrete routing, late generation completion, provider selection/removal, missing endpoints, privacy, and session refresh.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 110.22 kB (gzip 30.38 kB) and `dist/studio-foundation.css` 3.39 kB (gzip 1.11 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. Static evidence only, not new runtime browser evidence.

## Checks run at the head

At implementation head `ed666322b`, `npm test`, typecheck, build, Node model-source tests, and legacy strict-page checks pass. Final repository gates, claim wording, local policy, diff/status, and exact delivery commit/tree are rerun after this report and recorded on the PR before queueing. Fresh exact-head review and green GitHub checks remain required.

No test used real browser data, credentials, a real model, network, pod, deployment, or production publish. Modern Web Guidance informed native dialog (`dialog`) and progress (`progress`) behavior; `navigation-drawer` informed the labeled `aria-controls`/`aria-expanded` trigger without another modal/focus system. README distinguishes native dialog semantics from pending real-browser focus verification.

## Skills used

Brainstorming against the approved design, writing/executing plans, isolated worktrees, TDD with failing-first cycles, systematic debugging, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute or paid resource was used.

## Still open

Full real-browser feature parity, worker/offline/mobile/accessibility checks, one-time browser-data migration, and cutover remain. The pre-existing R4G1 assertion and real File System Access manual/headed verification gap remain out of scope. The unrelated roadmap sync failure remains disclosed. #3500 remains ready for parity/migration; production publication is a separate owner decision.

## Plain summary

The Studio now has a complete typed core-workflow library with 100 passing tests, including regressions for every finding from the first independent review. It preserves model, file, session, generation, and privacy identity while calling existing browser modules. The live page has not been replaced; this is not a deployment or full-parity claim.
