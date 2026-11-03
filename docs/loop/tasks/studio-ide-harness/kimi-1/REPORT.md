# REPORT: Mac-inspired Studio IDE and simplified UOR/API connections

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), with follow-up [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569); both issues remain open. Order: [ORDER.md](ORDER.md). Design: [design](../../../../superpowers/specs/2026-11-02-studio-ide-harness-design.md). Plan: [plan](../../../../superpowers/plans/2026-11-02-studio-ide-harness.md). Original base: `85f634af30809c998cc3dbd830366c85be214934`. Merged forward from `origin/main` `7394c344d` and `b6ebf7002745f1c9ec5c7de223bc9f9ae66009e3` before final delivery checks; exact delivery-head evidence is posted on the PR.

## What did not work

- The frozen session ended after the three implementation/documentation commits and before the REPORT, browser delivery evidence, exact-head review, or PR. Those five commits (including the two planning commits) were intact and clean; no uncommitted source needed reconstruction.
- `origin/main` advanced by DeepSeek's #3581 before verification. The branch merged that commit forward without force-push; no Studio files conflicted.
- The checkout had no generated `web/studio/pkg`, so the page truthfully reported the engine unavailable. This visual/source unit did not build a model bundle or invoke a model, API provider, Test, Send, or real file.
- Browser automation needed a fresh element snapshot after DOM changes, and the responsive device setup required explicit orientation/dimensions. One Connections click was not treated as evidence; the actual ⌘3 destination was then verified directly. These were automation adjustments, not product changes or claimed failures.
- No real model/API trajectory, MCP connectivity, desktop stdio bridge, real disk access, typed-library cutover, production publish, or trained-chat capability was exercised or is claimed. #3569 and #3500 remain open for their broader units.

## What changed

The recovered five-commit unit:

- Applies a Mac-inspired native visual system to the existing plain-JS Studio: system typography, calmer opaque surfaces, softer radii, compact native controls, at least 24px targets, and visible keyboard focus in dark/light themes and responsive layouts.
- Simplifies Connections around **UOR Chat — experimental** through the existing supported non-discrete stack/WASM path, **Custom API providers**, and a closed **Artifact diagnostics** region. Original buttons, listeners, `data-action` values, readiness checks, storage, and truthful capability labels are preserved.
- Renames user-facing Models navigation to Connections while preserving internal `models`/`model` keys and compatibility IDs. No engine, worker, tokenizer, `studio_api.js`, model artifact, MCP behavior, dependency, publisher, deployment, `.github/**`, or typed-library cutover was added.
- Includes the ORDER, roadmap item, design, implementation plan, and focused actual-page tests for computed visual treatment, source regrouping/listener preservation, readiness, and navigation labels.

During verification, the modern-web accessibility check exposed that the custom mobile drawer updated ARIA/scrim/`inert` without moving focus. A new regression failed first (`expected '' to be 'navAgent'`). `setChrome` now moves focus to the first destination on mobile open and returns it to Menu on scrim, destination, or Escape close. All 216 tests and the real-browser open/close check pass after that narrow repair. The recovered commits, merge-forward, ORDER/REPORT, and this focus repair form the delivery.

## Measurements and checks

At the merged verification head before this REPORT commit:

- `npm test`: **27 test files, 216 tests passed, 0 failures**.
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 123.07 kB (gzip 33.77 kB), `dist/studio-foundation.css` 3.57 kB (gzip 1.14 kB), uncommitted.
- `node --test assets/js/model_sources.test.mjs`: **12 tests passed, 0 failures**.
- `node --check assets/js/studio_files.js` and `node --check assets/js/studio_ui.js`: PASS.
- `node verify_studio_page.mjs --strict-router`: PASS for `index.html` and `geometric_prime_router_webapp.html`; `legacyResidual: []`, `failures: []`.
- `node verify_studio_page_load.mjs --strict-router`: PASS. Studio: 12 page-load requests, 0 external; router: 2 requests, 0 external. Planted external detection passed on inline, blob-worker, and late channels.
- Combined executed unit/model-source tests: **228 passed, 0 failures**.

Repository gates, claim wording, local policy, exact-head review, and GitHub checks are recorded at the exact delivery head below after this REPORT is committed.

## Real browser evidence

The built-in browser exercised the actual local page, with no real credentials/files, external endpoints, or model request:

- **Desktop 1440×900:** Agent, Workspace, Connections, and Guide; dark and light themes; visible keyboard-focus ring; primary UOR Chat source; custom providers; collapsed diagnostics; no-source Send disabled.
- **Demo workflow:** opened the six-file demo tree, opened two editor tabs, edited the CSS buffer without saving to disk, used it as the Agent prompt, and attached `web/studio/index.css`. The inspector/composer showed one file and the explicit local-WASM context limitation.
- **Mobile 393×852:** Agent, Workspace, Connections, and Guide rendered without horizontal page overflow. Navigation started closed; the labeled backdrop appeared when opened; Escape, backdrop click, and destination selection each closed it, with the Menu left collapsed.
- The page-load trap independently passed its planted-request self-test. Browser presentation and source readiness are visual/non-model evidence, not trained-chat, live interoperability, real API/disk, production-publish, or typed-parity claims.

## Storage and preservation

- Three older clean, merged Kimi worktrees were removed only after their ignored/untracked notes and scratch were moved byte-preserved into `local/kimi/retired-worktrees/`; their PRs were #3476, #3436, and #3468. Other labs' worktrees and scratch were untouched.
- Guarded `uor-hygiene --apply` removed those worktrees and idle build output, freeing about 2 GB. Two additional known idle Cargo target directories (`local/target`, `local/pod-provider-target`) were cleaned for 652.0 MiB of regenerable build output. Nothing unique was deleted and the Trash was not emptied.
- Free space reached 30,131,724 KiB (about 30.1 GB decimal / 29 GiB) before the Studio suite, restoring the project's 30 GB minimum. Generated `node_modules`, `dist`, and the SDD ledger remain uncommitted and will be handled by the normal post-merge cleanup.

## Checks run at the head

- `UOR_LAB=kimi CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target/kimi cargo run -q --locked -p uor-r4-harness --bin uor-harness -- loop gates --full`: **11/11 GREEN**.
- `uor-harness loop pr-check --local`: the first run identified the new-item merge-base anchor and two missing REPORT headings. The ORDER now names the existing parent #3500 plus the new follow-up #3569 without hiding scope, and the required headings are present. The corrected rerun is recorded in the final PR receipts.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` and `git diff --check`: final results are recorded in the PR receipts.
- The 228 Studio/model-source checks and both page-load layers listed above ran before the REPORT commit. After the focus regression, the focused navigation test and the complete 216-test suite, typecheck, and `studio_ui.js` syntax check all passed again; the real-browser focus transition was also rechecked.

## Skills used

Recovery and working discipline, isolated worktrees, plan execution, test-driven development, systematic debugging, solution-integrity, scope/context guards, built-in-browser verification, modern web guidance, and fresh exact-head delivery review. No GPU, paid resource, real credential, real user data, or model compute was used.

### Modern Web Guidance

- **`navigation-drawer`**: I retained the existing sibling-scrim fallback rather than introducing a new popover/stacking system, and verified that Escape, backdrop, and destination dismissal all converge on the same collapsed/inert state ("replace `::backdrop` with a sibling element styled identically"). Focus is no longer left outside the custom overlay.
- **`accessibility`**: I added failing-first and real-browser checks for entering/restoring focus ("Move focus to the overlay"), retained native buttons and `:focus-visible` outlines, and kept minimum targets at 24×24 CSS pixels ("Enforce the target size of elements using `min-block-size` / `min-inline-size`").

## Exact delivery head

Pending this REPORT commit, fresh exact-head review, protected PR checks, and merge-queue delivery. This section is updated with those results; an open PR is not reported as merged.

## Still open

- #3569 remains open for real remote MCP behavior and the later desktop bridge. No decorative MCP panel or simulated discovery was added.
- #3500 remains open for typed-library parity/migration/cutover. The Preact library stays unmounted.
- Production publication remains separate. The current trained M1 artifact remains next-token prediction, not a conversational product.

## Plain summary

The recovered Studio unit is verified: it has a coherent native-style IDE presentation, a simplified Connections workflow, safe demo editing and attachments, working desktop/mobile navigation, and truthful model/API limits. All executed checks passed. No new serving or model capability is claimed.
