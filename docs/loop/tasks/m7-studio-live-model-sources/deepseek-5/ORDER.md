# ORDER: m7-studio-live-model-sources, deepseek-5 (workable AI studio v2)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: 1f83dd4b2. Owner, 30 Oct 2026: "that design looks fairly janky and the menu's do not make a lot of sense because they are not organized or labeled very well... can you please improve every piece of this so it will become a workable AI studio (sort of like the deepseek harness you are using) that can actually help build itself (both the API and the studio) - I don't want you to do work on the API yet... but you could make the actual studio much better than it is".

## Goal

Turn the Studio into a workable AI coding-agent workspace: clearly labeled, organized navigation (single left sidebar: Agent, Workspace, Models, Guide; permanent Sessions section), organized menus and models page, command palette grouped by workflow with real shortcuts, and real browser-side workflows - a File System Access workspace with a file tree, editor tabs and save; agent context attachments, Build task templates and message actions; session rename/duplicate/export/delete; settings and a guide. Presentation and browser workspace only: the engine, worker, model-source manager, API client and all verifier contracts are untouched (Claude owns the API; no API implementation here).

## Cancel probe

- Origin/main 1f83dd4b2 is the live console design with the four verifiers passing; the Studio is static HTML/CSS/JS with File System Access available in Chromium-class browsers and no backend.
If a required workflow cannot be done without a server, ship the browser-only version with a clear fallback and record the limit.

## Files in scope

`web/studio/index.html`, `web/studio/index.css`, `web/studio/assets/js/studio_ui.js`, new `web/studio/assets/js/studio_files.js` (workspace/editor), `web/studio/README.md`, `docs/products/web-client.md`, `docs/roadmap/items/m7-studio-live-model-sources.md`, `docs/loop/tasks/m7-studio-live-model-sources/deepseek-5/**`.

## Done when

At the final head:
- exactly four labeled primary destinations (Agent, Workspace, Models, Guide) with no duplicate icon-only nav or duplicate controls; Sessions is a permanent sidebar section with New session, search, rows with title/preview/time and a working row menu (rename/duplicate/export JSON/delete with an in-app confirm); old element ids remain present (hidden compatibility controls) so scripts/verifiers pass;
- Workspace works: Open folder... (File System Access API) with a file tree, import-files fallback, editor tabs with active-tab-driven content/name/language, dirty dot, Save (Cmd/Ctrl+S)/Save as, line/col and saved state; no silent overwrite;
- Agent works: context attachments (current file / workspace files / pasted text) with chips, token/size hint and remove; Build task templates (>=5 incl. Studio UX improvement brief and API integration checklist) that fill structured prompts; message actions Copy/Insert into editor/Save as file/Retry; explicit Local vs API privacy mode;
- Models page is organized into Local engine (offline) / Published models / API providers (remote) / Research artifacts (collapsed, experimental) with consistent metadata and no clipped cards; Guide and Settings exist and are labeled (theme, reduce motion, editor font/tab/wrap, data clear, About); status bar is one coherent labeled bar, no legacy duplication, AA contrast;
- command palette is grouped (Navigation/Sessions/Workspace/Agent/Models/Build/Interface/Help) with label + description + only-implemented shortcut, scrolls with the footer pinned, and advertises only real shortcuts (Cmd/Ctrl+K, ", 1-4, B, I, S, Shift+S, Esc, ?);
- mobile 390px: labeled nav drawer, no horizontal overflow, two-row labeled status bar, usable composer/editor/agent; desktop looks like a professional AI studio and passes the ux-critique acceptance gates;
- the four verifiers pass with assertions untouched (static scan, runtime page-load trap external [], worker, desktop contract), no external assets/URLs/requests; no engine/worker/model_sources/studio_api/crates/verifier edits; claim-wording, git diff --check, roadmap check/render --check, loop pr-check --local PASS; five required checks policy: pass.

## Stop and ask if

A verifier would need weakening, an existing element id/handler would be dropped, an external request would enter the page, or a workflow needs a server (do not add one; document the fallback).
