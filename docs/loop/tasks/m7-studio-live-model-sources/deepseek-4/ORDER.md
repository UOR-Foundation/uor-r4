# ORDER: m7-studio-live-model-sources, deepseek-4 (agent-console redesign)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: 21157ca7d. Owner, 30 Oct 2026: "that studio kind of sucks, anyway you can make it way cooler looking? more like claude code or codex desktop".

## Goal

Redesign `web/studio` into a modern coding-agent desktop console: warm graphite + ember palette with a designed light theme, mono-forward micro-labels, a 46px command bar (wordmark, workspace breadcrumb, project chip, command search, view switch, theme/new), a 54px icon rail, collapsible sessions sidebar with environment card, central transcript with a quiet console hero and terminal-frame composer, a full-width Model providers page, a compact scrollable context inspector, and a mono status bar with engine/model/privacy/storage/clock. Add an accessible Cmd/Ctrl+K command palette (keyboard nav, focus trap, `#command-palette` deep link), persisted theme and chrome toggles, responsive single-column mobile with zero horizontal overflow at 390px, and restrained motion with `prefers-reduced-motion`. Preserve every existing page id, handler, status marker, boot contract and offline/local-first honesty; no external assets or URLs; no engine/worker logic changes.

## Files in scope

`web/studio/index.html`, `web/studio/index.css`, `web/studio/assets/js/studio_ui.js` (new), `web/studio/README.md`, `docs/products/web-client.md`, `docs/roadmap/items/m7-studio-live-model-sources.md`, `docs/loop/tasks/m7-studio-live-model-sources/deepseek-4/**`.

## Cancel probe

- Origin/main (21157ca7d) is the live page; it has every current id/handler and the four verifiers pass at the base. The redesign is visual/interaction only; the engine, worker, model-source manager and API client are untouched.
If a functional contract cannot be preserved without weakening a verifier, stop and report.

## Done when

At the final head:
- every pre-existing element id and handler is preserved (no missing ids), the status markers and data-studio-ready boot contract remain, and the page kept/removed lists in the desktop verifier are unchanged;
- `verify_studio_page.mjs` (static zero-external load scan), `verify_studio_page_load.mjs` (runtime trap; index requests all same-origin, external []), `verify_studio_worker.mjs` and `verify_desktop_client.mjs` pass with assertions untouched;
- a headless functional run loads the bundled synthetic demo, generates tokens in the new composer, opens the palette with Cmd/Ctrl+K and runs a view switch, toggles theme, switches to Model and loads the trained M1 model via the published card, and returns the deterministic top-8 prediction; 390px viewport has scrollWidth == innerWidth;
- screenshots (desktop dark/light, palette, mobile, model, editor) are attached to the report and reviewed by the Lead;
- roadmap check/render --check current, claim-wording and git diff --check pass, loop pr-check --local PASS, five required checks carry `policy: pass`.

## Stop and ask if

A verifier must be weakened, an id/handler must be dropped, an external asset or URL would enter the page, or the mobile viewport cannot be made overflow-free.
