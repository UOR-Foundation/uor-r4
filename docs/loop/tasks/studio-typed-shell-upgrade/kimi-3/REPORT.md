# REPORT: deliver the live Studio IDE experience

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Design: [design](../../../superpowers/specs/2026-11-02-studio-ide-experience-design.md). Plan: [plan](../../../superpowers/plans/2026-11-02-studio-ide-experience.md). Base: `35d7d7b9e225cc97280d379c185d034b451151c2`. Exact delivery-head commit/tree evidence is posted on the PR.

## What did not work

- The initial desktop was serviceable but had weak hierarchy and a marketing-like Agent empty state. The new welcome panel presents useful source status and actions without implying a serving model.
- No-source and discrete-M1 Send appeared available. One shared readiness predicate now disables the button and shared send path, while supported non-discrete synthetic continuation and valid HTTP(S) API configuration remain available.
- Demo workspace existed only behind a hash/public API, could replace dirty work, restored as an unlabeled partial state, and fabricated a dirty edit. The shared demo path now refuses dirty work, restores/labels demo state, and never fabricates unsaved edits.
- Initial phone rendering put quick actions behind the composer and squeezed attachments into a vertical strip. Mobile quick actions now fit in two rows and the composer stacks full-width controls and attachments.
- The first full suite exposed an older page-hook fixture missing the new `chatReadiness` function. Including it restored both actual-page tests; no production behavior was weakened to accommodate the fixture.
- Policy initially reported only the required REPORT missing. Other checks were advisories; the report now exists.
- No real model, real API response, real user file, real disk access, production publish, or typed-library cutover is claimed. Real File System Access and full typed-library parity/migration remain later units.

## What changed

- Added shared source readiness for no source, discrete M1, supported non-discrete synthetic continuation, and valid HTTP(S) API configuration. Blocked sends return false and direct users to Models.
- Replaced the generic Agent empty state with distinct unavailable, API-ready, and local-ready welcome panels and quick actions: Open Models, Open Workspace, Explore demo workspace, and New session.
- Exposed the existing demo tree through the Agent action. Dirty tabs block replacement; demo tabs restore as demo with topbar/status/inspector labels, and no automatic dirty timer remains.
- Added one labeled mobile scrim, synchronized `aria-expanded`, main-content inertness, scrim click, destination selection, and Escape dismissal.
- Applied a restrained IDE visual system to the existing shell: clearer panel surfaces, readable type, stronger active states, useful welcome cards, editor tabs, model cards, dialogs, status fields, and responsive layouts in dark/light themes.

No engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, real browser-data migration, external asset, runtime dependency, or `.github/**` change was made. The typed library remains unmounted; this is a live legacy-page visual/non-model unit, not a typed cutover.

## Measurements

- **146 Vitest tests pass across 23 files**, including 6 live Agent readiness/empty-state tests, 3 actual demo workspace tests, and 1 actual mobile navigation test. **9 Node model-source tests pass. 155 tests total, 0 failures.**
- `npm run typecheck`: PASS, no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` 117.69 kB (gzip 32.31 kB) and `dist/studio-foundation.css` 3.57 kB (gzip 1.14 kB), uncommitted.
- `node verify_studio_page.mjs --strict-router`: PASS, both legacy strict pages clean. Modified `studio_ui.js`, `studio_files.js`, and `model_sources.js` syntax checks pass.
- Repository gates: 11/11 GREEN. Claim wording and local policy PASS with disclosed D76 scope/ORDER/FILEMAP advisories.

## Real browser evidence

Built-in browser checks exercised the live page, not only component fixtures:

- Desktop 1440×900: Agent welcome, Workspace, Models, Guide, demo tree, editor tabs, attachment state, and no-source Send disabled.
- iPhone 14 Pro 393×852: all four quick actions fit above the composer; attachments remain full width; drawer opens with a dimmed backdrop and closes via scrim and Escape. No horizontal page overflow was visible in these viewports.
- Dark and light themes preserve readable contrast.
- Demo workflow: Explore demo → open both tabs → edit `index.css` → switch to Agent → attach it. The inspector shows one context file and Send remains disabled.
- API-ready state: a dummy `https://example.invalid/v1` provider was saved and selected. The empty state reads “Agent connected — external baseline mode.” No Test or Send was invoked, so no endpoint was contacted.

These are live visual/non-model checks, not model, API, real-disk, production-publish, accessibility-certification, or typed-library parity claims.

## Checks run at the head

Focused tests were written failing-first for readiness, demo guard/restoration, and navigation. Tasks 1–3 have committed red/green records in the SDD ledger. Full Studio checks, repository gates, claim wording, local policy, exact-head review, and green GitHub checks precede queueing. PR uses `References #3500`; #3500 remains open for later parity/migration/cutover.

## Skills used

Brainstorming, design and implementation planning, isolated worktrees, TDD, systematic debugging, modern web guidance, verification before completion, and independent exact-head delivery review. No model compute, paid resource, real credential, or real user data was used.

## Still open

This delivery does not provide real model/API/disk trajectories or mount the typed component library. Production publication and full typed-library parity/migration remain later units; #3500 stays open.

## Plain summary

The live Studio now looks and behaves like a coherent IDE harness: navigation and sessions at left, useful work in the center, context at right, and command/status chrome around it. Users can explore and edit a demo workspace before a serving model exists. Chat availability is honest, the demo is safe, and desktop/mobile presentation is verified. No serving-model capability is claimed.
