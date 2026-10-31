# UOR-R4 Studio shell upgrade — 31 October 2026

## Intent and why

Rebuild the Studio's browser shell so its model, file, session, and privacy states are coherent by construction, and so future work on the Studio does not require another full-page reskin. The owner approved the recommended typed-shell direction: Preact + Vite, with the existing WASM worker, model-source/API adapters, offline behavior, and publication pipeline preserved.

The measured defect is fragmented state ownership. Independent DOM writers report model/file/session status without one authoritative lifecycle store. On the live page, a model source can show `ACTIVE` while the inspector says `No model loaded`, and Workspace can show `Saved` beside `No file open`. Two presentation redesigns improved the shell's appearance but did not make these states impossible. The current implementation has about 11,000 lines across HTML, CSS, and JavaScript without a component build system.

## Goals

- One typed lifecycle store for model sources, loaded artifacts, files, sessions, and privacy mode.
- Every badge, button, status line, empty state, and enabled action derives from that store.
- A clear primary workflow: open workspace → attach context → choose model → run → review/edit/save.
- Separate source selection from loaded model state; separate local next-token prediction from chat.
- Responsive, keyboard-complete, higher-contrast UI with useful empty states and no duplicate status bars.
- A component architecture that supports independent migration and behavioral tests.
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

- Work only in the isolated `kimi/studio-shell-upgrade` worktree and protected PR path.
- Allowed implementation scope: `web/studio/src/**`, shell entry/config/package/test files, and, only when the new shell is ready for cutover, its publication staging helpers and documentation.
- `web/studio/assets/js/model_sources.js`, `studio_api.js`, `uor_model_worker.js`, `uor_tokenizer_artifact.js`, and the engine's worker/WASM boundary remain unchanged. A typed adapter may call their existing public interface.
- `crates/**`, model artifacts, `.uor-models/**`, `.github/**`, Claude paths, and generated `pkg/` contents are out of scope.
- Existing `uor.studio.*` browser data must be read and migrated non-destructively, with migration tests and no real credentials in tests.
- Pin npm dependencies in the lockfile. `node_modules/` and build output remain uncommitted.
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

- Model source: `unselected → selected → loading (indeterminate while the delegated fetch/hash/tokenizer/parse runs) → ready | error`. No byte progress or verification substate is shown unless the existing adapter reports that real event.
- Loaded artifact: identity, schema, bytes, hash verification, engine capability, and load error. A selected source cannot display `ACTIVE` unless its own artifact is the loaded ready artifact.
- File state: `no folder → no open file → clean/dirty → saving → saved | error`. Save and prompt actions are disabled when no file is open; demo/local-draft state is labeled explicitly.
- Session state: `new → active → generating → complete | stopped | error`, with local/API privacy mode derived from the actual selected source.
- Privacy: local-only until a remote API is actually active. A configured provider alone does not imply that prompts leave the device.

All UI status derives from this store. Direct DOM status writes outside components are not part of the new architecture.

### 2. Components

- `AppShell`: labeled primary navigation, one top-level status region, responsive drawer.
- `ModelPicker`: source selection, download/hash progress, loaded artifact and capability state.
- `WorkspaceEditor`: folder/file selection, tabs, dirty/save state, empty-state action guards.
- `AgentComposer`: context attachments, local-predictor versus API-chat modes, generation state.
- `SessionList`: non-destructive local session operations and migration.
- `Inspector`: on-demand details rather than a permanent duplicate status pane.
- `Guide` and `Settings`: actual implemented controls only.

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

- Type checking and the production Vite build succeed.
- State-transition tests include: source selected but not loaded; second source selected while one is ready; hash/load failure; no file open; dirty then saved; empty session; generation stopped/error; local versus remote privacy.
- Component tests assert the visible status and enabled/disabled actions for those states.
- Existing worker/page checks pass at their declared legacy/new pages, including synthetic trajectory and bit-identical save/restore where applicable.
- Real-browser tests cover model loading, file editing/saving, agent modes, keyboard/focus, mobile 390 px, and zero external page-load requests.
- Tests include negative cases; a failed build/check cannot be reported as a pass.
- Visual inspection covers desktop, mobile, light/dark, model-ready/error, and workspace empty/dirty states.
- Final review confirms no forbidden model/API/worker/Claude changes entered the diff.

## Builder-check decisions

The independent builder check identified seven choices that materially affected planning. They are settled as follows; these are lead decisions, not owner-approved feature removals.

1. **Full feature parity at cutover.** The new shell must retain the current command palette, six build-task templates, message copy/insert/save/retry, session rename/duplicate/export/delete, save-as/download, import-files fallback, API provider add/test/edit/remove, research section, editor preferences, and data clearing, in addition to the named components and primary workflow. No feature is silently dropped. Each gets a behavioral check or an explicit legacy-parity record.
2. **Development and legacy paths.** Vite root is `web/studio`, source lives under `src/`, and output is the uncommitted `web/studio/dist/`. The current `index.html` remains the development legacy page until cutover. At cutover it moves to `legacy.html` as a local-only check artifact and is not published as a second application. Old checks target `legacy.html`; new checks target the built `index.html`. The publisher stages `dist/*` at `/studio/` while preserving generated `pkg/` and other required assets. `scripts/studio/build-and-publish.sh` may run `npm ci && npm run build`; `.github/**` remains untouched.
3. **One-time copy-forward persistence.** On first boot, the new shell reads existing `uor.studio.*` keys once, validates and migrates them into new versioned keys, and writes an idempotent migration marker. Old keys remain byte-identical for rollback; there is no dual-write and no post-cutover synchronization promise. Plaintext provider credentials are never logged or included in test fixtures.
4. **Truthful loading progress.** The frozen `model_sources.js` promise boundary does not expose separate download/hash stages or byte progress. The store therefore uses indeterminate `loading` and shows only events that really occur. It never invents `verifying` or percentage states, re-hashes artifacts, or treats legacy DOM text as a second source of truth.
5. **Pre-existing R4G1 assertion left untouched.** `crates/uor-r4-wasm-router/tests/bdd.rs` already expects option markup absent from the current page. The crate and assertion are out of scope and the failure is pre-existing. The new page will not add hidden compatibility markup. Release QA must not be reported as green; this known stale assertion is disclosed in the handoff rather than repaired or concealed.
6. **File-save evidence.** Automated browser verification uses import-files plus download-save and a stubbed directory handle for dirty/save transitions. The real File System Access picker requires a user gesture and remains a named manual/headed verification gap, not an automated pass.
7. **Explicit privacy states.** `localWASM` means prompts stay in the browser worker; `loopbackAPI` means prompts go to a service on the same device; `privateNetworkAPI` means prompts go to a LAN/private host; `remoteAPI` means prompts go to the configured remote endpoint. Loopback/private/remote classification comes from the parsed endpoint hostname, with tests for localhost, `127.0.0.1`, `::1`, private IPv4/IPv6, and public hosts. No generic "local" badge covers API traffic.

## Builder latitude

The builder may choose component file layout, exact type names, accessible icon treatment, design tokens, and whether to split delivery into a foundation PR followed by workflow PRs. The builder may not change the selected state architecture, preservation boundaries, dependency class, offline/privacy requirements, full-feature-parity requirement, or cutover requirements without returning to the owner.

## Delivery decision

Deliver in small protected PRs, beginning with the typed store, tests, and shell foundation. Do not replace the live entry point or publish until the complete migration checks pass and a fresh exact-head review approves the cutover. The generated roadmap views, issue status, and production deployment remain separate from the implementation unit.
