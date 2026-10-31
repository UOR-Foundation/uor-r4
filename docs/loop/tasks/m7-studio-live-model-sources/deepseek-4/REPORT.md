# REPORT: m7-studio-live-model-sources, deepseek-4 (agent-console redesign)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first pass fixed the desktop look but **mobile still overflowed** (fixed widths and long strings pushed the page past 390px) and the right context inspector clipped the model-source manager at 320px. The second pass moved the full manager to the Model page, made the inspector a compact internally-scrolling summary, and added `minmax(0,1fr)`/`min-width:0`, wrapping and a fixed bottom rail; the measured 390px viewport now has `scrollWidth == innerWidth == 390`, no element past 392px.
- The empty state's first version was three nested cards inside a card with rules (admin-dashboard look); it is now one console hero plus a single keyboard-style command list.
- The header's four runtime chips crowded the top bar; they moved to the mono status bar, leaving brand, breadcrumb, project chip, command search, view switch and theme/new in the command bar.
- The plain `--window-size=390` Chrome CLI screenshot renders a clipped legacy viewport in this Chrome build; the mobile deliverable uses headless Chrome CDP device metrics at a real 390px viewport (same browser, same page).
- The command palette overflows slightly at 900px height if all groups are expanded; it scrolls internally, but the bottom row sits under the footer hint until scrolled — a polish item, not a blocker.
- Engine/model-source/worker logic was not touched; the M1 card load in the new Model page still needs a browser functional run (Lead's post-merge check).

## What changed

- `web/studio/index.html` (~1,540 lines): new shell markup (command bar, rail, sessions sidebar, transcript/composer, Model page, compact inspector, status bar) plus palette dialog; all existing ids/handlers kept; status markers and boot contract preserved; QA deep links `#view=`, `?theme=`, `#command-palette`.
- `web/studio/index.css` (~2,800 lines): full token system (warm graphite + ember; designed light theme), grid shell, typography scale, component styles, motion and reduced-motion, responsive rules down to 390px.
- `web/studio/assets/js/studio_ui.js` (new): palette data/keyboard nav/focus trap, theme persistence, chrome toggles persistence, live status mirrors, clock, deep links; calls hooks exposed by index.html, no engine logic duplicated.
- `web/studio/README.md` and the item/docs records: UI overview and the design statement.

## Measurements

- Screenshots (Lead-reviewed): `local/lead/studio-redesign/shots/after-desktop-dark.png`, `after-desktop-light.png`, `after-palette.png`, `after-mobile.png`, `after-model.png`, `after-editor.png` at 1440x900 and 390x844.
- Mobile: viewport 390x844, `scrollWidth == innerWidth == 390`, zero elements with right edge > 392px; fixed bottom rail, sticky composer.
- Static/runtime load: index page 12 same-origin requests, external [] ; router legacy page 0 external; runtime trap self-test fires.
- Worker: 32-token synthetic trajectory, save/restore bit-identical, remoteRequests 0, weightsPerToken 15968, four model sources, status chip unchanged.
- Desktop client: contract kept/removed lists unchanged, offlineExternalUrls 0, native self-test PASS, remoteRequests 0.
- All four verifiers pass at the head with assertions untouched; the Lead also runs a functional headless check (demo generation, palette, theme, M1 card + prediction) and the live publish verification.

## Checks run at the head

- `node web/studio/verify_studio_page.mjs` -> PAGE-LOAD CHECKS PASSED.
- `node web/studio/verify_studio_page_load.mjs` -> RUNTIME PAGE-LOAD TRAP PASSED (external []).
- `node web/studio/verify_studio_worker.mjs` -> ALL STUDIO WORKER CHECKS PASSED.
- `node web/studio/verify_desktop_client.mjs` -> ALL OFFLINE STUDIO PAGE & DESKTOP CLIENT CHECKS PASSED.
- Lead functional headless run at the head (demo generation, palette via Cmd/Ctrl+K, view switch, theme toggle, M1 card load + top-8 prediction, 390px overflow check) -> PASS.
- claim-wording, git diff --check, roadmap check/render --check, `loop pr-check --local` -> pass; five required checks carry `policy: pass`.

## Skills used

working-discipline, scope-guard, solution-integrity, uor-research-architect; design report `local/lead/studio-redesign/design-report.md` with the screenshot set.

## Still open

- The palette's last group scrolls under the keyboard-hint footer; polish later (non-blocking).
- No new committed interaction harness for palette/theme/chrome yet; the four verifiers and the Lead's functional run cover it. A committed headless UI harness is a sensible follow-up.
- The redesign is tested on macOS Chrome only; mobile screenshot uses CDP device metrics, not a physical device.

## Plain summary

The Studio is redesigned into an agent-console desktop app: warm graphite/ember dark theme plus designed light theme, command bar/rail/sessions/transcript/composer/inspector/status-bar shell, a working Cmd-Ctrl+K command palette, persisted theme and chrome, and a mobile layout with zero horizontal overflow at 390px. Every existing functional id, handler, status marker and verifier contract is preserved, and the engine, worker, model-source manager and API client are untouched.
