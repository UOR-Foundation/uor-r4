# Design: unified Apple-like web and Tauri harness IDE

## Intent and why

The owner likes the new dark conversation interface and wants its simplicity applied to a complete desktop harness, not a prettier chat panel hiding the old IDE elsewhere. The product should feel like a straightforward coding desktop: talk to the model, choose a project, edit files, inspect changes, run an explicitly approved command, connect models/MCP, and adjust settings. Web and Tauri share the same interface and behavior wherever the platform permits it.

The current Studio has no active Tauri calls. The older Tauri model worker uses an obsolete byte-mod tokenizer and is not a parity path. The existing desktop backend already supplies granted-folder file I/O, Git status, guarded terminal execution, model management and durable memory; these are the capabilities to expose, not replace with mocks.

## Goals

- One canonical Apple-like design across every view, modal, drawer, menu and responsive breakpoint.
- A small set of obvious workflows: Chat, Files, Changes, Terminal, Connections and Settings; sessions and command palette remain easy to reach.
- One frontend shared by Pages, other modern browsers and a Tauri desktop app.
- Exact native chat parity through the already verified `uor-chat` OpenAI-compatible path.
- Real local folder, Git, guarded terminal, durable-session and MCP workflows with explicit errors and tests.
- A buildable macOS arm64 Tauri app bundle containing the native sidecar and runtime assets, with no committed binaries or secrets.

## Non-goals and anti-goals

- No useful-conversation, visual-understanding, production-readiness or M3/M7/M8 completion claim.
- No second desktop-specific interface, legacy panel skin, fake terminal/MCP button, silent network fallback, paid provider or public API tunnel.
- No unrestricted folder picker, page-enabled terminal, arbitrary page-launched MCP executable, token in logs/storage, or weakening of admission, context, command or artifact checks.
- No claim that the app bundle is one self-contained native binary: it is one Tauri app package with a managed `uor-chat` sidecar and desktop backend.

## Approaches considered

1. **Shared adaptive frontend, verified sidecar, and native host capabilities (chosen).** One design system and workflow map for every platform. `uor-chat` supplies exact current OpenAI behavior; the desktop backend supplies host-only operations. This maximizes reuse and avoids the obsolete tokenizer bridge.
2. **Separate desktop-only interface.** Rejected because it doubles maintenance and guarantees visual/behavioral drift, exactly what the owner rejects.
3. **WebView around the old IDE plus decorative native buttons.** Rejected because it preserves the old presentation and cannot prove folders, terminal or MCP work.
4. **Revive the old native model worker.** Rejected because its byte-mod tokenizer is not the current stack-bound BPE/OpenAI path.

## Canonical interface and design system

- Replace the accumulated theme overrides with one canonical stylesheet and component vocabulary: system type, black/graphite surfaces, blue accents, generous spacing, rounded cards/bubbles/controls, visible focus, forced-colors support, and 16px mobile form text. Preserve existing element IDs and engine protocols; do not append another competing skin.
- Chat stays a centered conversation with rounded messages and floating composer. Sessions open in a simple drawer rather than a permanent inspector.
- Files has a readable tree, clean tabs and editor, with Open folder as the primary empty action. Changes shows branch/status and a bounded diff. Terminal shows exact command, working directory, consent state and output; disabled/unapproved states are explicit.
- Connections separates local models, custom OpenAI providers and MCP, with the native runner guide visible. Settings groups appearance, data/storage and About. The command palette exposes the same real actions and no dead shortcuts.
- Every modal, session row menu, build-template menu, diagnostics disclosure, toast and mobile drawer follows the same tokens and focus rules. Desktop and 390px mobile views are inspected per view, not only chat.

## Native/chat architecture

- The Tauri host builds and launches the pinned `uor-chat-arm64-apple-darwin` sidecar with `--stack`, `--tokenizer`, `--serve 127.0.0.1:8000`, `--studio-root` and `--token-stdin`. The launch token is generated in memory and never logged or persisted.
- The sidecar serves the current Studio and authenticated OpenAI API on the same origin, including its existing ephemeral bootstrap. This preserves exact tokenization, streaming, usage, image decomposition and context errors rather than reimplementing them in the old desktop worker.
- A second loopback channel serves desktop RPC on `127.0.0.1:1420`, with a separate in-memory token. It accepts only the exact Studio origin and required preflight/JSON requests, returning the same exact CORS origin. No public address, wildcard origin, HTTP Pages connection or token persistence is allowed.
- Runtime and sidecar paths come from packaged resources or the development tree as appropriate. `src-tauri/binaries/` is generated and ignored; a build helper verifies the pinned sidecar hash before packaging. `bundle.active` becomes true only after the real package path is verified.

## Native host operations and trusted consent

- Add `tauri-plugin-dialog` (or the maintained Tauri v2 dialog API if already present) for actual host-owned folder selection and terminal confirmation. This new dependency is required because a web-page confirmation boolean is not trusted consent. Dialog permissions are scoped to the main window; if unavailable, the affected action fails closed.
- Folder selection returns a canonical path to the host, which alone calls `native_grant_folder`. File listing/read/write and durable-store paths retain granted-root and symlink checks. Browser File System Access remains the browser fallback.
- Changes adds bounded `git diff` beside the existing safe status invocation, with hooks/fsmonitor disabled and output caps. No implicit stage, commit, push or destructive Git action.
- Terminal enablement and each exact command require native confirmation. Existing allowlisted direct execution, metacharacter refusal and granted-folder confinement remain. The UI displays command/cwd/consent/output and never treats a web boolean as approval.
- Durable session save/restore uses the existing object-store path and remains synthetic protocol evidence, not an M3 qualification.

## MCP architecture

- **Remote Streamable HTTP (shared web/Tauri):** user-entered URL and optional credential, explicit Connect action, protocol initialize/initialized, `tools/list` and bounded `tools/call`. Support JSON and SSE responses, session/protocol headers, timeout/size limits and actionable CORS/auth/transport errors. Credentials are used only for the configured request and are not logged.
- **Native stdio (Tauri only):** user-visible executable, args and working directory require native approval before spawn. No shell string is executed. Use direct process spawn with an executable allowlist, granted working directory, bounded frames/timeouts and mutex-serialized JSON-RPC. Implement initialize/initialized, `tools/list` and `tools/call`, plus explicit unsupported/error states and process cleanup. Browser stdio is reported as unavailable, never simulated.
- MCP tools are shown with names/descriptions and only invoked after an explicit user action. No model is allowed to call a tool autonomously in this unit.

## Verification and delivery

- Failing-first tests cover the canonical view map, native folder/terminal consent, exact CORS/preflight, sidecar bootstrap, bounded Git diff, remote JSON/SSE MCP, stdio MCP lifecycle, and package resource/hash validation.
- Existing Studio, Node, worker/page/WASM, native loopback, vision, xtask and desktop controls remain green. New tests exercise the real shipped JS/Rust paths, not duplicate mock implementations.
- Real browser checks cover every view/submenu at desktop and mobile sizes. Real native GUI checks cover chat streaming, folder choice/file save, Git diff, terminal consent, durable resume and both MCP transports using fixtures. The packaged `.app` is launched, not merely compiled.
- Independent architecture and exact-head reviews precede protected merge. Fresh main, Pages runtime identity, public sidecar hashes and final free space are verified. No binary, corpus, secret or development-only file enters the source or Pages commits.

## Explicit limits and follow-up

The first package target is macOS arm64; other platforms are not claimed. MCP support is bounded to the listed protocol operations, not every MCP transport, notification, resource, prompt, sampling or elicitation feature. Advanced agent autonomy, arbitrary MCP executables, automatic Git writes, signed updater distribution and production model quality remain follow-up work.
