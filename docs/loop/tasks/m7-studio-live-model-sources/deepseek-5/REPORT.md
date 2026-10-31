# REPORT: m7-studio-live-model-sources, deepseek-5 (workable AI studio v2)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md).

## What did not work, first

- The 31 Oct console redesign was judged "fairly janky" with menus that were "not organized or labeled very well": an icon-only rail, a duplicate view switch, flat unlabeled palette groups, a scratchpad editor with no workspace, list-only sessions, and a status bar that failed contrast. An independent UX critique (local/lead/ui-v2/ux-critique.md) confirmed the defects and required merging Files+Editor into one Workspace, a permanent Sessions section, one control per action, only real shortcuts, AA contrast, and 11 hard acceptance gates.
- The first v2 pass still had a broken mobile layout, duplicated status-bar text from the legacy footer, inconsistent workspace tabs (tabs did not drive the editor), an empty Published models section and clipped model cards, and a session empty state in screenshots. Round 2 fixed these (details/measurements below).
- File System Access API is Chromium-only and cannot be fully exercised by a headless picker; the Workspace ships a file-import fallback and download-based save, and the limits are documented.

## What changed

- New labeled navigation and IA: single left sidebar with Agent, Workspace, Models, Guide (shortcuts 1-4), workspace selector + Open folder, permanent Sessions section (New session, search, rows with preview/time, row menu rename/duplicate/export/delete with in-app confirm), footer Settings/Help; old ids remain as hidden compatibility controls.
- Workspace: File System Access folder tree + import fallback, editor tabs driven by the active file, dirty/saved state, Save/Save as, line/col; clean no-file empty state.
- Agent: workflow-oriented empty state, Attach context (current file / workspace files / pasted text) with chips and token hint, Build task templates, message actions Copy/Insert into editor/Save as file/Retry, explicit Local WASM vs API privacy labels.
- Models page: Local engine (offline) / Published models / API providers (remote) / Research artifacts (collapsed experimental), consistent metadata, no clipped cards.
- Guide and Settings views; single labeled status bar; grouped command palette with descriptions and real shortcuts only; keyboard map implemented (Cmd/Ctrl+K, ", 1-4, B, I, S, Shift+S, Esc, ?).
- Mobile: labeled drawer navigation, condensed two-row status bar, zero horizontal overflow, usable composer/editor/agent.
- Records: item follow-up, web-client/README, ORDER/REPORT; no generated view hand-edited.

## Measurements

- Screenshots (Lead-reviewed; local/lead/ui-v2/shots/): after-agent.png, after-agent-light.png, after-agent-with-attachments.png, after-palette.png, after-workspace.png, after-models.png, after-guide.png, after-settings.png, after-mobile.png (+ drawer/session-menu states as captured).
- Mobile: 390x844 viewport, document/body scrollWidth == innerWidth == 390, no element past 392px.
- Functional headless run (Lead): nav switches all four destinations; palette opens/closes and groups scroll; sessions create/rename/duplicate/export/delete; workspace import + tabs + edit + save-as download; attachments add/remove + template fill; settings theme/motion persist; guide opens; page-load external [] before interaction; M1/synthetic model flows unchanged.
- Four verifiers pass with assertions untouched: static scan PAGE-LOAD CHECKS PASSED; runtime trap external [] (index same-origin only); worker ALL PASSED (remoteRequests 0); desktop ALL PASSED (kept/removed lists unchanged).
- CI/loop: five required checks policy: pass; pr-check --local PASS; this report's host line carries the exact commands and results at the head.

## Checks run at the head

- node web/studio/verify_studio_page.mjs -> PAGE-LOAD CHECKS PASSED.
- node web/studio/verify_studio_page_load.mjs -> RUNTIME PAGE-LOAD TRAP PASSED (external []).
- node web/studio/verify_studio_worker.mjs -> ALL STUDIO WORKER CHECKS PASSED.
- node web/studio/verify_desktop_client.mjs -> ALL OFFLINE STUDIO PAGE & DESKTOP CLIENT CHECKS PASSED.
- Lead functional headless E2E for the v2 workflows (nav, palette, sessions, workspace import/tabs/save-as, attachments/templates, settings, guide, mobile overflow) -> PASS (details in the final answer).
- claim-wording, git diff --check, roadmap check/render --check, loop pr-check --local -> pass; five required checks carry policy: pass.

## Skills used

working-discipline, scope-guard, solution-integrity, uor-research-architect; UX critique local/lead/ui-v2/ux-critique.md; screenshots local/lead/ui-v2/shots/.

## Still open

- File System Access write-back is Chromium-only; other browsers use import + download, documented in the Guide. Directory handles are in-memory only.
- Hooked local model emits token ids/repetition (synthetic demo is not language); the trained M1 trunk serves top-8 next-token prediction only. API mode is wired to any OpenAI-compatible endpoint but the API work itself is Claude's; no server was added.
- No committed browser-workflow harness yet; the Lead's local headless E2E covers the acceptance gates and a committed harness is a sensible follow-up.
- macOS Chrome tested; screen-reader/real-hardware review not run.

## Plain summary

The Studio is now a labeled, organized AI workspace rather than a themed shell: four clear destinations with a permanent sessions sidebar, a real file workspace with tabs and save, an agent with context attachments, build templates and message actions, organized model sections, settings, a guide and a working command palette, in both themes and on mobile without overflow. The engine/API layer is untouched, every page contract and verifier still passes, and the API integration itself remains Claude's work.
