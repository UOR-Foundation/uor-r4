# Studio Default Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the plain hosted Studio URL mount the neutral workbench while keeping explicit legacy access and all existing runtime boundaries.

**Architecture:** Add a small same-document `window.location.replace` redirect before persistence or legacy application code can run. It adds `typed=1` only when neither an explicit typed nor legacy selection exists, preserves other query parameters and the URL hash, and leaves the existing typed bootstrap unchanged.

**Tech Stack:** Existing HTML/vanilla JavaScript, Preact/Vite, Playwright, Node test tooling, Bash publication scripts, protected GitHub delivery.

**Spec:** `docs/loop/tasks/studio-ide-harness/kimi-16/ORDER.md`

**Execution:** Inline implementation in one isolated worktree, followed by fresh exact-head adversarial review, protected merge, fresh-main verification, and guarded publication. The user's request for the plain hosted UI to match the native workbench is the cutover authorization.

## Global Constraints

- No model/API/engine/worker/tokenizer, storage migration, Tauri, native-consent, dependency, `.github/**`, or public-service change.
- Keep `?typed=1` mounting and early persistence behavior unchanged. Explicit `?legacy=1` or `?typed=0` selects the legacy interface instead.
- Do not claim full Cowork runtime, useful model behavior, a hosted PTY, or live R4 API integration. Static browser limitations remain explicit.
- Test the real page navigation before editing production HTML; no mock-only acceptance or post-hoc test.

## Review Focus

- Empty query: enter the typed workbench, not the legacy conversation page.
- Explicit `legacy=1` or `typed=0`: retain the old interface without a redirect.
- Additional query parameters and `#theme=light`: survive the default redirect exactly.
- Existing `typed=1`: do not create a navigation/reload loop or a second model worker.
- Native `index.html?typed=1`: remains unchanged and needs no routing or storage edit.

---

### Task 1: Default-route regression and minimal redirect

**Files:**
- Create: `docs/loop/tasks/studio-ide-harness/kimi-16/ORDER.md`, `docs/loop/tasks/studio-ide-harness/kimi-16/REPORT.md`, `docs/superpowers/plans/2026-11-09-studio-default-cutover.md`.
- Modify: `web/studio/index.html`, `web/studio/tests/e2e/shell.spec.ts`, `web/studio/tests/e2e/visual.spec.ts`, `web/studio/README.md`.
- Modify after actual publication: `docs/integration/current-state.md`, `docs/products/web-client.md`, `docs/products/harness-parity.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/studio-ide-harness.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`.

**Interfaces:**
- Consumes: existing `typedPreviewRequested(search)` and early persistence activation, both of which already recognize `typed=1`; the typed Vite publisher and existing Playwright configuration.
- Produces: plain `/studio/` redirects to `?typed=1`; `?legacy=1` and `?typed=0` retain the legacy page; other parameters and hash are preserved. No TypeScript, model, storage, or native API signature changes.

- [ ] **Step 1: Write the failing test**

Add a real Playwright case to `web/studio/tests/e2e/shell.spec.ts`:

```ts
await page.goto('/?publish=cutover#theme=light');
await expect(page).toHaveURL('/?typed=1&publish=cutover&theme=light#theme=light');
await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
await expect(page.locator('#app')).toBeHidden();
await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
```

Also cover `?legacy=1` and `?typed=0`: each must keep the legacy `#app` visible and must not activate the typed root.

- [ ] **Step 2: Run the test to verify it fails**

Run: `npx playwright test tests/e2e/shell.spec.ts --grep "default URL enters the typed workbench"`.
Expected: FAIL because the plain root retains the legacy URL and `#app` remains visible. Legacy fallback cases pass before and after the fix.

- [ ] **Step 3: Implement the early default redirect in `web/studio/index.html`**

Before any stylesheet, persistence, or legacy application script runs, return immediately for `typed=0`, `typed=1`, or `legacy=1`. Otherwise translate an existing theme hash into the existing `theme` query when no explicit theme selection is present, delete the old typed value, and call `window.location.replace(window.location.pathname + '?typed=1&' + params.toString() + window.location.hash)`. Do not edit `bootstrap.ts`, `lifecycle_persistence.js`, the native URL, or the publisher. Legacy screenshot fixtures explicitly use `?legacy=1` so their original coverage remains intact.

Update `web/studio/README.md` to describe default typed mounting and the explicit legacy fallbacks without claiming a full runtime cutover.

- [ ] **Step 4: Run focused and full checks**

Rerun the focused case, then `npm test`, `npm run typecheck`, `npm run test:e2e`, and `python3 -m unittest scripts.tests.test_typed_runtime scripts.tests.test_tauri_runtime`. Require every result to pass. Build `web/studio/dist`, verify the typed staged output and direct WASM checks, and inspect real default/legacy desktop and mobile navigation with zero runtime errors.

- [ ] **Step 5: Commit and deliver**

Commit the failing-test evidence separately from the production fix, then commit REPORT and verification evidence references at the final head. Obtain fresh exact-head approval, required checks, protected queue acceptance, actual merge, and fresh-main verification.

- [ ] **Step 6: Publish and verify the actual default page**

From exact fresh main, run the existing typed publisher/allowlist checks, publish only after they pass, preserve the previous gh-pages head for rollback, and verify plain `/studio/` at desktop and mobile sizes. Require the neutral workbench, one typed root, working explicit legacy fallbacks, no raw-TypeScript or runtime errors, and an explicit static-terminal limitation. Update product/current records and evidence; do not claim the shared agent runtime or live R4 API.
