# Studio Typed Lifecycle Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a tested typed lifecycle store, narrow adapters, fixture-backed migration reader, and additive model-source progress/operation hooks without changing the live Studio page.

**Architecture:** A small framework-independent TypeScript store owns selection, resident artifacts, operations, API tests, files, sessions, and per-generation privacy. A narrow adapter calls the existing model-source/worker APIs, while two additive hooks in `model_sources.js` expose structured progress and suppress stale operations. Pure fixture-backed migration functions describe copy-forward plans without touching real browser data.

**Tech Stack:** TypeScript, Preact, Vite, Vitest, jsdom, existing browser JavaScript modules, Node's built-in test runner.

**Spec:** `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`

## Global Constraints

- Do not mount/replace `index.html`, change `index.css`, invoke the publisher, deploy, or read/write real browser data.
- Keep `studio_api.js`, `uor_model_worker.js`, `uor_tokenizer_artifact.js`, and the engine byte-identical.
- `model_sources.js` may gain only structured progress events and optional operation metadata; existing callers retain behavior. Do not scrape DOM or duplicate download/hash/tokenizer/persistence logic.
- Selecting a new artifact revokes run readiness. Previous resident artifacts are diagnostics only and cannot receive new runs.
- Pin source and privacy per generation. API selection is not artifact readiness or a reachability claim.
- `modelSources.v1` remains the live provider/compatibility key and is never copied into a second secret store.
- Pin npm dependencies. `node_modules/` and `dist/` stay uncommitted. No production components or cutover work.

## Review Focus

- Unknown byte totals must never produce percentages; local/bundled loads without progress events remain indeterminate (Task 3).
- A superseded or failed replacement must never restore readiness or select the previous source (Tasks 2 and 3).
- Switching sources during generation must not relabel the active generation (Task 2).
- API selection/test states must not fabricate artifact readiness or reachability (Task 2).
- `modelSources.v1` must remain the only provider-secret store, while migratable keys preserve raw rollback data and tolerate malformed/unknown fields (Task 4).

---

### Task 1: Build/test scaffold and public entry point

**Files:**
- Create: `web/studio/package.json`, `web/studio/package-lock.json`, `web/studio/tsconfig.json`, `web/studio/vite.config.ts`, `web/studio/src/main.ts`, `web/studio/src/main.test.ts`

**Interfaces:**
- Consumes: approved Preact/Vite/TypeScript/Vitest/jsdom dependency classes.
- Produces: `npm test`, `npm run typecheck`, `npm run build`, and a library entry that later exports lifecycle, adapter, and migration APIs.

- [ ] **Step 1: Write the failing entry test**

`main.test.ts` imports `foundation` from `./main.js` and asserts it is an object. Initially `./main.ts` and the package configuration do not exist.

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/main.test.ts`
Expected: FAIL because the entry/module/package setup is missing.

- [ ] **Step 3: Add the minimal scaffold and entry**

Use ESM, strict TypeScript, `moduleResolution: "Bundler"`, `target/lib: ES2022` plus `DOM`/`DOM.Iterable`, `types: ["vite/client"]`, and Vitest `environment: "jsdom"`. Vite library entry is `src/main.ts`, output `dist`, target `es2022`, no page input. Export an empty `foundation` object for this task. Pin the resolved dependency tree in `package-lock.json`.

- [ ] **Step 4: Run entry test, typecheck, and build**

Run: `npm test -- src/main.test.ts && npm run typecheck && npm run build`
Expected: PASS, no TypeScript errors, and a generated library bundle in uncommitted `dist/`.

- [ ] **Step 5: Commit**

Commit the scaffold, lockfile, entry, and passing entry test.

### Task 2: Lifecycle state and per-generation privacy

**Files:**
- Create: `web/studio/src/state/lifecycle.ts`, `web/studio/src/state/lifecycle.test.ts`
- Modify: `web/studio/src/main.ts`

**Interfaces:**
- Consumes: Task 1 scaffold.
- Produces: `LifecycleStore` with `getState()` and `dispatch()`, artifact operation transitions, API test transitions, file transitions, generation transitions, and `classifyEndpoint(input?: string): "localWASM" | "loopbackAPI" | "privateNetworkAPI" | "remoteAPI"`.

State requirements:
- `LoadOperation = { operationId: number; sourceId: string }`.
- Artifact phases: `unselected`, `selected`, `downloading`, `finalizing`, `ready`, `error`.
- `DownloadProgress = { loaded: number; total: number | null }`; percent exists only when `total > 0`.
- Selecting revokes run readiness immediately. Ready requires matching current operation and source. Stale events are ignored.
- API selection has `testStatus: "untested" | "testing" | "passed" | "failed"`, never an artifact phase.
- Files support no folder/no open file, clean/dirty, saving, saved, and error; save is disabled without an open file.
- Each generation pins its source and privacy at start and retains them through complete/stopped/error.

- [ ] **Step 1: Write failing lifecycle tests**

Cover selected-not-ready; known/unknown totals; finalization not ready; second source superseding and stale completion; failed replacement disabling runs; API selected-untested and independent test result; no-file save refusal; dirty/save transitions; generation source/privacy remaining pinned while the picker changes; endpoint classification for localhost, `127.0.0.1`, `::1`, private IPv4/IPv6, and public hosts.

- [ ] **Step 2: Run lifecycle tests to verify they fail**

Run: `npm test -- src/state/lifecycle.test.ts`
Expected: FAIL because the lifecycle store does not exist.

- [ ] **Step 3: Implement the minimal store and endpoint classifier**

Use a small reducer/store, no state-library dependency. Ignore stale operation IDs, derive readiness only from a current ready event, and keep generation source/privacy snapshots immutable. Classify loopback before private ranges and never infer privacy from configuration alone.

- [ ] **Step 4: Run lifecycle tests and export the API**

Run: `npm test -- src/state/lifecycle.test.ts`
Expected: all lifecycle tests PASS. Export the lifecycle types/store/classifier from `src/main.ts`.

- [ ] **Step 5: Commit**

Commit the store and tests.

### Task 3: Additive model-source progress/operation hooks and adapters

**Files:**
- Modify: `web/studio/assets/js/model_sources.js`
- Create: `web/studio/assets/js/model_sources.test.mjs`
- Create: `web/studio/src/adapters/modelSourceAdapter.ts`, `web/studio/src/adapters/modelSourceAdapter.test.ts`
- Modify: `web/studio/src/main.ts`

**Interfaces:**
- Consumes: Task 2 `LoadOperation`, lifecycle transitions, and existing `ModelSources` public API plus worker `model_loaded`/`operation_error` messages.
- Produces:
  - JS `LOAD_PROGRESS_EVENT = "uor-model-load-progress"` with `{ operationId?, loaded, total, phase }`, where `total` is byte count or `null` and `phase` is `downloading` or `finalizing`.
  - Optional JS `{ operationId, sourceId }` metadata on `loadPublished`, `loadFromUrl`, `loadLocalFile`, and `loadBundled`; stale completion returns `{ stale: true }` without selecting/persisting/emitting source-changed.
  - TS `ModelSourceAdapter` and `createModelSourceAdapter()` forwarding operations and structured progress, with worker readiness/errors mapped to lifecycle transitions.

- [ ] **Step 1: Write failing JS hook and TypeScript adapter tests**

Use a minimal valid UORLUT01 fixture and mocked fetch/response/worker. Assert real byte events, unknown totals without percentage, finalization, operation metadata, stale completion having no selection effect, and legacy calls without metadata retaining behavior. Adapter tests assert the existing public methods are called, not reimplemented, and only worker-ready marks the artifact ready.

- [ ] **Step 2: Run both tests to verify they fail**

Run: `node web/studio/assets/js/model_sources.test.mjs && npm test -- src/adapters/modelSourceAdapter.test.ts`
Expected: FAIL because the progress event/operation metadata and adapter are missing.

- [ ] **Step 3: Add the two JS hooks and narrow TS adapter**

Forward the existing internal byte callback as structured events without computing percentages for unknown totals. Emit one indeterminate finalization per load. Operation IDs are monotonic per caller; newer operations supersede older ones. Legacy callers without metadata follow the original path. The adapter only forwards calls/events and worker readiness.

- [ ] **Step 4: Run both tests and export the adapter**

Run: `node web/studio/assets/js/model_sources.test.mjs && npm test -- src/adapters/modelSourceAdapter.test.ts`
Expected: all JS hook and adapter tests PASS. Export the adapter types/factory from `src/main.ts`.

- [ ] **Step 5: Commit**

Commit the additive hooks, adapter, and tests.

### Task 4: Fixture-backed migration reader and copy-forward plans

**Files:**
- Create: `web/studio/src/persistence/migration.ts`, `web/studio/src/persistence/migration.test.ts`
- Modify: `web/studio/src/main.ts`

**Interfaces:**
- Consumes: a `Storage`-compatible `getItem(key)` fixture only.
- Produces:
  - `MIGRATION_KEYS`, covering `conversations.v1`, `activeConversation.v1`, `editor.v1`, `modelSources.v1`, `settings.v1`, `ui.v2`, legacy `ui.v1`, `attachments.v1`, `tabs.v2`, `recent.v2`, and `theme` under `uor.studio.`.
  - `readMigrationSnapshot(storage)`, returning raw input plus parsed values/defaults and malformed-input diagnostics without secret logging.
  - `createCopyForwardPlan(snapshot)`, a deterministic idempotent plan with `modelSources.v1` excluded as the live compatibility key and no writes or migration marker.

- [ ] **Step 1: Write failing migration tests**

Cover every key, unknown fields, malformed JSON, missing values, idempotent plans, unchanged raw rollback data, and no second copy of provider secrets. Use only synthetic sentinel values, never real credentials.

- [ ] **Step 2: Run migration tests to verify they fail**

Run: `npm test -- src/persistence/migration.test.ts`
Expected: FAIL because the migration reader/plan functions do not exist.

- [ ] **Step 3: Implement the pure migration reader and plan**

Read each key once, preserve raw strings, tolerate unknown fields, and report malformed values without throwing. `modelSources.v1` has a live compatibility role and no destination key or duplicate secret payload. No real storage writes or marker exists in this unit.

- [ ] **Step 4: Run migration tests and export the API**

Run: `npm test -- src/persistence/migration.test.ts`
Expected: all migration tests PASS. Export the keys/types/functions from `src/main.ts`.

- [ ] **Step 5: Commit**

Commit the migration reader and tests.

### Task 5: Full verification and delivery record

**Files:**
- Modify: `docs/loop/tasks/studio-typed-shell-upgrade/kimi-1/REPORT.md`
- No production changes.

- [ ] **Step 1: Run the complete foundation suite**

Run: `npm test && npm run typecheck && npm run build && node web/studio/assets/js/model_sources.test.mjs && node web/studio/verify_studio_page.mjs --strict-router`
Expected: all tests PASS, no TypeScript errors, library build succeeds, and both strict pages remain clean.

- [ ] **Step 2: Run repository gates and local policy**

Run: `uor-harness loop gates --full` and `uor-harness loop pr-check --local`
Expected: 11/11 gates GREEN and policy PASS with no undisclosed failure.

- [ ] **Step 3: Record exact evidence and fresh review**

Write REPORT with failures/limits first, exact test counts and commands, size, no browser-data/deployment changes, and remaining workflow/cutover work. Obtain fresh adversarial exact-head review, green GitHub checks, protected merge, and fresh-main verification. #3500 stays ready for the next workflow unit.
