# UOR-R4 Studio shell upgrade — rebased and rescoped 1 November 2026

## Intent and why

Rebuild the Studio's browser shell so its model, file, session, and privacy states are coherent by construction, and so future work on the Studio does not require another full-page reskin. The owner approved the recommended typed-shell direction: Preact + Vite + TypeScript, with the existing WASM worker, model-source/API adapters, offline behavior, and publication pipeline preserved.

The reviewed defect is fragmented state ownership. Independent DOM writers report model/file/session status without one authoritative lifecycle store. Static markup starts with `Saved` beside `No file open`, and selected model source, loaded artifact, and mirrored status are computed separately. Two presentation redesigns improved the shell's appearance but did not make contradictory states impossible.

## Rebased review and scope decision

- Current `main` is `381ae4d98b0a5100d241a268d283629ab9b5fd1f`. Its Studio source is byte-identical to the original design base; there is no newer implementation to adopt. The source is 11,068 lines across HTML, CSS, and JavaScript with no component build system. `node web/studio/verify_studio_page.mjs --strict-router` passes its static zero-external-load check.
- DeepSeek's completed v2 and its historical plain-JS constraint stay untouched. This is a new owner-approved scope, tracked separately, not a reopening or rejection of that work.
- Keep the selected Preact + Vite + TypeScript approach. The defect is still architectural, but delivery is phased: a foundation unit first, workflow migrations second, full-parity cutover last. There is no implementation in this design PR.
- Correct the original loading assumption: `model_sources.js` already emits real downloaded-byte counts, totals when known, and percentages. The new store must expose them. Only finalization stages without events are indeterminate.
- Current persistence is part of the parity contract, not an optional migration detail. Existing keys are inventoried below, including the current `ui.v2` and older `ui.v1` forms, and no real browser data is read or written in the foundation unit.

## Goals

- One typed lifecycle store for model sources, loaded artifacts, files, sessions, and privacy mode.
- Every badge, button, status line, empty state, and enabled action derives from that store.
- A clear primary workflow: open workspace → attach context → choose model → run → review/edit/save.
- Separate source selection from loaded model state; separate local next-token prediction from chat.
- Real byte-download progress when reported, with indeterminate finalization and stale-load rejection when another source supersedes an operation.
- Responsive, keyboard-complete, higher-contrast UI with useful empty states and no duplicate status bars.
- Semantic landmarks and controls, native modal dialogs, and native progress elements rather than custom focus/status mirrors.
- A component architecture that supports phased migration and behavioral tests without requiring a big-bang replacement.
- Self-hosted static build output with no runtime CDN or new page-load network requests.

## Non-goals

- No model training, model-mechanism, API, worker, tokenizer, or engine changes.
- No changes to model artifacts, acceptance bars, M1–M8 status, or Claude's active files/worktrees.
- No new backend, hosted service, external account, analytics, or runtime third-party request.
- No production deployment or modification of `.github/**`.
- No attempt to make M1 appear to be a conversational assistant.

## Anti-goals and failure conditions

- A visual redesign that leaves contradictory model/file/session status possible.
- Replacing the current shell before the new workflows pass their behavioral and runtime checks.
- Clearing or rewriting existing browser sessions, drafts, provider settings, or credentials during migration.
- Weakening existing engine/offline checks, deleting failing assertions, or hiding unsupported behavior behind compatibility controls.
- New runtime dependencies on external assets or a required build-time service beyond npm on a developer machine.

## Constraints and preservation boundaries

- Work only in an isolated lab worktree and deliver each unit through its protected PR path.
- Foundation implementation scope is limited to `web/studio/src/**` and shell entry/config/package/test files. Entry-point, publication-helper, and documentation changes wait for their later units.
- `web/studio/assets/js/model_sources.js`, `studio_api.js`, `uor_model_worker.js`, `uor_tokenizer_artifact.js`, and the engine's worker/WASM boundary remain unchanged. A typed adapter may call their existing public interface.
- `crates/**`, model artifacts, `.uor-models/**`, `.github/**`, Claude paths, and generated `pkg/` contents are out of scope.
- Existing `uor.studio.*` browser data must be migrated non-destructively. The foundation unit uses fixtures only: no real browser-data read/write, migration marker, or credentials in tests. Real copy-forward migration is a later cutover unit.
- Pin npm dependencies in the lockfile when implementation begins; do not install dependencies for this design-only PR. `node_modules/` and build output remain uncommitted.
- No production publish. Preview/cutover requires its own verified PR and the existing owner-controlled publish path.
- If `main` advances, merge it forward and re-check affected Studio behavior rather than overwriting other labs' work.

## Approaches considered

### A. Zero-dependency custom elements and a central store

Keep the current static application and introduce custom elements with idempotent setup/teardown and one lifecycle store. This minimizes build changes and dependencies, but leaves more manual component composition and type checking to conventions. It is the fallback if the owner later rejects new build dependencies.

### B. Typed Preact + Vite shell — selected

Use Preact for a small component runtime, Vite for a static self-hosted build, and TypeScript for explicit lifecycle contracts. Use a small typed event store with `useReducer`/context rather than adding a state-library dependency. Vitest + jsdom covers state transitions and components; the existing Node/Chrome harness covers real browser behavior without adding a large browser-framework dependency.

This is the best fit for a substantial upgrade because it makes lifecycle mismatches compile-time and behavior-test concerns while keeping the runtime small and the existing engine adapter intact. The cost is a new npm build toolchain and migration effort, which the owner accepted when choosing this direction.

### C. Native/shared-product shell

Build a shared web/Tauri product architecture now. Rejected for this unit because it entangles the shell with unfinished M7, desktop, and backend/API work and increases the chance of touching Claude's active model path.

## Selected architecture

### 1. Typed lifecycle store

Define explicit transitions and invariants for:

- Model source: `unselected → selected → downloading (bytes known/unknown) → finalizing (indeterminate) → ready | error`. Byte counts and percentages come only from the existing adapter's real `onProgress` callback. A zero/unknown total cannot produce a percentage, and download completion is not readiness.
- Every load operation carries a monotonically increasing operation id and its source id. Progress, errors, and completion from an older operation are ignored after another source supersedes it.
- Loaded artifact: identity, schema, bytes, hash verification, engine capability, and load error. A selected source cannot display `ACTIVE` unless the existing adapter confirms that its own artifact is loaded and ready for the current operation.
- File state: `no folder → no open file → clean/dirty → saving → saved | error`. Save and prompt actions are disabled when no file is open; demo/local-draft state is labeled explicitly.
- Session state: `new → active → generating → complete | stopped | error`, with local/API privacy mode derived from the actual selected source.
- Privacy: local-only until a remote API is actually active. A configured provider alone does not imply that prompts leave the device.

All UI status derives from this store. Direct DOM status writes outside components are not part of the new architecture.

### 2. Components

- `AppShell`: one `<main>`, named navigation landmarks, a responsive drawer, and one top-level status region rather than independent status mirrors.
- `ModelPicker`: source selection, loaded artifact and capability state, and a native `<progress>` element with an accessible name for real byte totals. Unknown totals and finalization use indeterminate text, never an invented percentage or verification stage.
- `WorkspaceEditor`: folder/file selection, tabs, dirty/save state, empty-state action guards.
- `AgentComposer`: context attachments, local-predictor versus API-chat modes, generation state.
- `SessionList`: non-destructive local session operations and migration.
- `Inspector`: on-demand details rather than a permanent duplicate status pane.
- `Guide` and `Settings`: actual implemented controls only. Modal settings/help/palette surfaces use native `<dialog>` with `.showModal()`, accessible names, and native close/focus behavior rather than manual focus traps, `inert` toggling, or redundant Escape handlers.
- Dynamic errors use an appropriate live region; ordinary loading progress does not repeatedly announce interstitial text.

### 3. Engine adapter

A narrow TypeScript interface wraps the existing public engine/model-source/API calls. It does not reimplement tokenization, model loading, generation, hashing, or persistence semantics. Existing engine/worker/API files remain byte-identical; behavioral checks compare the same synthetic model trajectory, save/restore, and offline boundaries.

### 4. Static build and migration

- Pin `preact`, `vite`, `typescript`, the Preact Vite preset, `vitest`, and `jsdom` in the lockfile. No state framework or UI kit.
- Vite uses the `/studio/` base and emits self-hosted static assets. Existing generated model/tokenizer/pkg paths stay unchanged.
- Develop and verify the new shell without replacing the current page. The old page remains available during migration; no live cutover occurs mid-verification.
- Preserve existing verifiers during migration. At cutover, add new behavioral selectors/contracts and keep or move the old checks to the retained legacy page rather than deleting or weakening them.
- Migrate `uor.studio.*` data through a versioned reader with defaults, unknown-field tolerance, and no credential logging.

## Backward chain and guards

For the finished upgrade to be true:

1. The new page must produce coherent status in every reachable state.
2. Therefore each status must derive from one lifecycle store, with tests for contradictory inputs and transitions.
3. Real model and file workflows must use that same store through the existing engine adapter.
4. Therefore adapter tests must prove the real existing worker/API calls are used, not a test reimplementation.
5. The new page must be safe to cut over.
6. Therefore legacy and new checks must both pass at their respective pages, with no assertion weakened, before the entry point changes.
7. Existing user data and offline/privacy behavior must survive.
8. Therefore migration, offline-request, and privacy-mode tests must pass before publication.

Cheap checks come before expensive ones: state-transition unit tests → component tests → static build → existing worker tests → real browser workflows → accessibility/mobile/offline checks → preview/cutover review.

## Required verification

- This design-only unit runs the existing static page-load check; it claims no new browser, worker, build, or accessibility result.
- The foundation unit proves type checking and the production Vite build, with state-transition tests for: source selected but not loaded; real byte totals and unknown totals; indeterminate finalization; a second source superseding an active load and stale completion; hash/load failure; no file open; dirty then saved; empty session; generation stopped/error; and local/loopback/private/remote privacy.
- Foundation adapter tests prove that the existing model-source/worker/API calls are used rather than reimplemented. Fixture-backed persistence tests cover every key below, malformed/unknown fields, idempotence, rollback preservation, and no credential logging. No real browser data is touched.
- Later component tests assert visible status, native progress/dialog behavior, accessible names, and enabled/disabled actions for those states.
- Existing worker/page checks pass at their declared legacy/new pages, including synthetic trajectory and bit-identical save/restore where applicable.
- Real-browser tests cover model loading, file editing/saving, agent modes, keyboard/focus, native-dialog focus return, mobile 390 px, and zero external page-load requests.
- Tests include negative cases; a failed build/check cannot be reported as a pass. Visual inspection covers desktop, mobile, light/dark, model loading/ready/error, and workspace empty/dirty states.
- Final review confirms no forbidden model/API/worker/Claude changes entered the diff.

## Builder-check decisions

The independent builder check identified choices that materially affected planning. They are settled as follows; these are lead decisions, not owner-approved feature removals.

1. **Full feature parity is the final cutover gate, not the first implementation unit.** The new shell must eventually retain the current command palette, six build-task templates, message copy/insert/save/retry, session rename/duplicate/export/delete, save-as/download, import-files fallback, API provider add/test/edit/remove, research section, editor preferences, and data clearing, in addition to the named components and primary workflow. No feature is silently dropped. Each gets a behavioral check or an explicit legacy-parity record before cutover.
2. **Development and legacy paths.** Vite root is `web/studio`, source lives under `src/`, and output is the uncommitted `web/studio/dist/`. The current `index.html` remains the development legacy page until cutover. At cutover it moves to `legacy.html` as a local-only check artifact and is not published as a second application. Old checks target `legacy.html`; new checks target the built `index.html`. The publisher stages `dist/*` at `/studio/` while preserving generated `pkg/` and other required assets. `scripts/studio/build-and-publish.sh` may run `npm ci && npm run build`; `.github/**` remains untouched. None of these entry-point or publisher changes belongs to the foundation unit.
3. **One-time copy-forward persistence.** The migration inventory is `conversations.v1`, `activeConversation.v1`, `editor.v1`, `modelSources.v1`, `settings.v1`, `ui.v2`, legacy `ui.v1`, `attachments.v1`, `tabs.v2`, `recent.v2`, and `theme`, all under `uor.studio.`. The foundation implements only a fixture-backed versioned reader with defaults and unknown-field tolerance. The later cutover unit reads those keys once, validates and migrates them into new versioned keys, and writes an idempotent migration marker. Old keys remain byte-identical for rollback; there is no dual-write or post-cutover synchronization promise. Plaintext provider credentials are never logged or included in test fixtures.
4. **Truthful loading progress.** The existing `model_sources.js` callback supplies real byte counts, totals when known, and percentages. The store uses those values while downloading, exposes indeterminate finalization for hash/tokenizer/worker stages that have no separate events, and never invents a percentage or re-hashes an artifact. Operation/source ids reject stale events, and only the adapter's actual ready result can mark an artifact loaded.
5. **Pre-existing R4G1 assertion left untouched.** `crates/uor-r4-wasm-router/tests/bdd.rs` already expects option markup absent from the current page. The crate and assertion are out of scope and the failure is pre-existing. The new page will not add hidden compatibility markup. Release QA must not be reported as green; this known stale assertion is disclosed in the handoff rather than repaired or concealed.
6. **File-save evidence.** Automated browser verification uses import-files plus download-save and a stubbed directory handle for dirty/save transitions. The real File System Access picker requires a user gesture and remains a named manual/headed verification gap, not an automated pass.
7. **Explicit privacy states.** `localWASM` means prompts stay in the browser worker; `loopbackAPI` means prompts go to a service on the same device; `privateNetworkAPI` means prompts go to a LAN/private host; `remoteAPI` means prompts go to the configured remote endpoint. Loopback/private/remote classification comes from the parsed endpoint hostname, with tests for localhost, `127.0.0.1`, `::1`, private IPv4/IPv6, and public hosts. No generic "local" badge covers API traffic. This changes presentation only, not endpoint behavior.

## Builder latitude

The builder may choose component file layout, exact type names, accessible icon treatment, and design tokens. The builder may not change the phased delivery, selected state architecture, preservation boundaries, dependency class, offline/privacy requirements, full-feature-parity requirement, or cutover requirements without returning to the owner.

## Delivery decision

Deliver in small protected PRs, each with its own ORDER at the merged item base:

1. **This design/scope PR only.** Rebase, review, correct, and register the phased item. No implementation or dependency installation.
2. **Foundation.** Add the typed store, narrow engine/persistence adapters, migration reader, fixture-backed tests, and build/test configuration. Do not mount or replace the page, change the publisher, or read/write real browser data.
3. **Core workflows.** Migrate AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings against the store and adapters. Keep the legacy page live and retain all current capabilities while their parity records are completed.
4. **Parity and migration.** Finish every feature/browser-data migration and all legacy/new behavioral, worker, offline, accessibility, and mobile checks. No production publication.
5. **Cutover.** Only after full parity and fresh exact-head approval, change the entry point and publisher. Production publication remains a separate owner decision through the existing controlled path; generated roadmap status is updated only on delivered evidence.
