# ORDER: unified Apple-like web and Tauri harness IDE

Lab: kimi. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `05c6ec70255ef82257165fa6dede603dce4f3fab`.

## Goal

Turn the conversation-first Studio into a simple, complete harness IDE without restoring any legacy panel styling. Keep one shared web frontend and one Tauri-packaged desktop app with the same Apple-like dark design. Expose ordinary desktop workflows—chat, sessions, local files, Git changes, guarded terminal, models, MCP and settings—using real existing or newly measured paths, not decorative controls.

## Design

[DESIGN.md](DESIGN.md). Use one adaptive frontend and the verified `uor-chat` sidecar for native/web parity. Do not revive the retired Tauri tokenizer bridge. Native dialogs own folder/terminal consent. Browser and desktop failure states stay explicit.

## Files in scope

- `web/studio/**`, `src-tauri/**`, and focused host/browser tests.
- `docs/products/desktop-client.md`, `docs/products/web-client.md`, `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/infra-desktop-client-tauri.md`, and this task folder.
- Build/package scripts under `scripts/studio/**` and `scripts/tests/**` only when required for the pinned sidecar or packaged-app verification.

## Done when

- Every view and submenu uses the same system-type dark Apple-like design; no old IDE panel treatment remains. Chat, Files, Changes, Terminal, Connections and Settings are responsive and straightforward.
- The shared page retains working WASM/OpenAI chat, sessions, history, long pastes and attachments, and adds real remote Streamable HTTP MCP configuration/initialize/tools list/call with honest CORS/auth errors.
- Tauri serves the same frontend through the verified `uor-chat` sidecar and authenticated desktop RPC, with separate in-memory tokens and exact origin/CORS limits. Native folder choice, file read/write, Git status/diff, host-confirmed terminal and durable sessions work end-to-end.
- Native stdio MCP uses an explicit host-approved executable/args/working directory, bounded JSON-RPC and initialize/tools list/call; arbitrary page-launched executables remain refused.
- The macOS arm64 Tauri bundle builds with generated/pinned sidecar artifacts and native dialogs. No binary or secret is committed or printed. Packaging and real GUI checks pass.
- Failing-first regressions, existing Studio/Node/Rust/desktop checks, independent exact-head review, protected merge, fresh-main verification and hosted-source verification pass. Final cleanup preserves evidence and at least 30 GiB free.

## Stop and report conditions

Stop before claiming useful conversation, visual understanding, production readiness, M3/M7/M8 completion, a single-binary app, stdio MCP without a real fixture, or unrestricted shell/folder access. Never replace host confirmation with a page-supplied boolean, weaken granted-folder or command allowlists, expose a token, or add a paid provider. If a required native dialog or packaging dependency is unavailable, report the exact blocker rather than shipping a fake control.
