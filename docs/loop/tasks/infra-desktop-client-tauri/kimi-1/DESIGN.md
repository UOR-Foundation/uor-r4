# Design: unified Apple-like web and Tauri harness IDE

## Intent and why

The owner likes the dark conversation interface and wants its simplicity applied to a complete desktop harness, not a prettier chat panel hiding the old IDE elsewhere. The product should support ordinary coding-desktop workflows: chat, sessions, local files, Git changes, guarded terminal, models, MCP and settings. Web and Tauri share one interface and behavior wherever the platform permits it.

The current Studio has no active Tauri calls. The older Tauri model worker uses an obsolete byte-mod tokenizer and is not a parity path. The existing desktop backend supplies granted-folder file I/O, Git status, guarded terminal execution, model management and durable memory; expose those capabilities rather than replacing them with mocks.

## Architecture review ruling

The first draft proposed a sidecar-served page plus a second desktop RPC server. Independent architecture review rejected that composition: `frontendDist` and sidecar origins conflict, two backends would split grants/approvals, sidecar chat has no durable engine snapshot, fixed ports add lifecycle risk, and the current CSP/verifiers contradict remote browser calls. It also found that the documented terminal allowlist is absent and that Git/terminal output needs stronger bounds.

The implementation therefore uses **Comparator B: one Tauri asset origin, one backend state, Tauri IPC for every privileged host action, and one `uor-chat` loopback sidecar reached only through a host-mediated streaming proxy.** No second HTTP server or second RPC token exists. This is an implementation decision from the review, not an owner-approved two-server design.

## Goals

- One canonical Apple-like design across every view, modal, drawer, menu and responsive breakpoint.
- Obvious workflows: Chat, Files, Changes, Terminal, Connections and Settings; sessions and command palette remain easy to reach.
- One frontend source shared by Pages, other modern browsers and Tauri.
- Exact native chat behavior through the verified `uor-chat` OpenAI-compatible path, mediated by Tauri rather than called from the page origin.
- Real local folder, Git, guarded terminal, local conversation history and MCP workflows with explicit errors and tests.
- A buildable macOS arm64 Tauri app bundle containing the native sidecar and runtime assets, with no committed binaries or secrets.

## Non-goals and anti-goals

- No useful-conversation, visual-understanding, production-readiness or M3/M7/M8 completion claim.
- No second desktop-specific interface, legacy panel skin, fake terminal/MCP button, silent network fallback, paid provider or public API tunnel.
- No unrestricted folder picker, page-enabled terminal, arbitrary page-launched MCP executable, token in logs/storage, or weakening of admission, context, command or artifact checks.
- No claim that the app bundle is one native binary: it is one Tauri app package with a managed sidecar and backend.
- No claim that sidecar chat resumes exact hidden engine state. Conversation text/history persists locally; `UR4SESS1` continuation remains limited to the older desktop-engine path and is not presented as sidecar resume or M3 evidence.

## Approaches considered

1. **Tauri asset origin, Tauri IPC, and host-mediated sidecar proxy (chosen).** One page origin and one state authority; native dialogs are Tauri commands. Only the loopback chat server remains, and the page never calls it directly.
2. **Two loopback servers with separate RPC token.** Rejected by architecture review for origin conflict, split state, CORS/token complexity and lifecycle risk.
3. **Separate desktop-only interface or decorative native buttons.** Rejected because it creates drift and cannot prove native workflows.
4. **Revive the old native model worker.** Rejected because its byte-mod tokenizer is not the current stack-bound BPE/OpenAI path.

## Canonical interface and design system

- Consolidate the current token-based Apple-like styling into one canonical component vocabulary rather than preserving competing legacy skins: system type, black/graphite surfaces, blue accents, generous spacing, rounded cards/bubbles/controls, visible focus, forced-colors support and 16px mobile form text. Preserve element IDs and engine protocols. Update existing theme tests deliberately rather than claiming they are untouched.
- Chat stays a centered conversation with rounded messages and floating composer. Sessions open in a simple drawer rather than a permanent inspector.
- Files has a readable tree, clean tabs and editor, with Open folder as the primary empty action. Changes shows branch/status and a bounded diff. Terminal shows exact command, working directory, consent state and bounded output; disabled/unapproved states are explicit.
- Connections separates local models, custom OpenAI providers and MCP, with the native runner guide visible and duplicate-guide prevention. Settings groups appearance, data/storage and About. The command palette exposes only real actions.
- Every modal, session row menu, build-template menu, diagnostics disclosure, toast and mobile drawer follows the same tokens and focus rules. Desktop and 390px mobile views are inspected per view, not only chat.

## One-origin native/chat architecture

- Tauri serves a staged runtime-only Studio from `frontendDist` under its asset protocol. No sidecar `--studio-root` or second frontend copy is used. `withGlobalTauri` stays enabled only for capability-scoped host commands.
- The host builds or reuses the pinned `uor-chat` sidecar, starts it on an available loopback port with `--stack`, `--tokenizer`, `--serve` and `--token-stdin`, and waits for an authenticated readiness check. Startup failure, collision and exit are explicit; normal app exit kills the child. No fixed-port success is assumed.
- A host-mediated Rust streaming proxy sends the bounded chat request to that sidecar with the in-memory token, reads its actual JSON/SSE response, and forwards events through a Tauri channel. The shared JS adapter turns those exact events into the existing OpenAI client stream. The page never contacts the sidecar directly, so the asset-origin and CSP restrictions cannot break it.
- The sidecar token remains in host memory and is not written to logs, URLs, localStorage or exported sessions. Any bootstrap provided to the shared model-source layer is an in-memory transport descriptor, not proof of direct browser reachability.
- Conversation text/history persists locally and can be exported. Exact sidecar `UR4SESS1` state export/resume is not claimed or faked.

## Native host operations and trusted consent

- Add `tauri-plugin-dialog` (or maintained Tauri v2 dialog API if already present) for actual host-owned folder selection and terminal confirmation. This dependency is required because a web-page boolean is not trusted consent. Dialog permissions are scoped to the main window; if unavailable, the affected action fails closed.
- A no-argument host command opens the native folder dialog, canonicalizes the selected path, and calls `native_grant_folder` on the same `TauriDesktopState` used by subsequent commands. The page never supplies a grant path or approval boolean. File I/O and durable-store paths retain granted-root and symlink checks.
- Changes adds bounded `git diff` beside the existing safe status invocation: no pager, no external diff/textconv, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, disabled hooks/fsmonitor, `status -z`, bounded streaming reads, and child termination at cap. No implicit stage, commit, push or destructive Git action.
- Terminal enablement and each exact command require native confirmation on the same backend. Add the documented exact-program allowlist before exposing the view, refusing interpreters such as `bash`, `sh`, `python3`, `osascript` and `env`, plus relative script execution. Direct execution, metacharacter refusal and granted-folder confinement remain. Add timeout and streaming output caps with explicit truncation, not unbounded `Command::output`.
- Existing object-store tests and the older desktop-engine durable-memory contract remain intact, but are not represented as sidecar chat resume or M3 qualification.

## MCP architecture

- **Remote Streamable HTTP in the browser:** user-entered URL and optional credential, explicit Connect action, protocol initialize/initialized, `tools/list` and bounded `tools/call`. Support JSON and SSE responses, session/protocol headers, timeout/size limits and actionable CORS/auth/mixed-content errors. Credentials are memory-only and never logged or stored beside other settings.
- **Remote HTTP in Tauri:** the host, not the asset-origin page, performs the request. A bounded native transport may use the system `curl` with sensitive configuration on stdin and no shell or token in argv. If that transport is unavailable, fail closed. No broad CSP or network exception is added.
- **Native stdio in Tauri:** exact executable path, args and granted working directory require native approval before direct spawn. This is host-approved exact-path execution, not a misleading interpreter allowlist. Bound frames/timeouts/process count, drain stderr, handle partial/coalesced frames, serialize JSON-RPC, and clean up children. Implement initialize/initialized, `tools/list` and `tools/call` with explicit errors. Browser stdio is reported unavailable.
- MCP names/descriptions/results render as text only, with an executable inert-markup regression. No autonomous tool invocation occurs in this unit.

## Packaging and resource integrity

- Stage a runtime-only Studio into `src-tauri/runtime/studio`; no `src`, tests, package configuration, declarations, verifier, README or `node_modules` enters the app or Pages publish.
- `src-tauri/binaries/` is generated and ignored. Use Tauri `externalBin` target-triple naming (`uor-chat-aarch64-apple-darwin`, not a double-triple name). Package the published pinned sidecar or pin a newly built artifact in a reviewed manifest; never call a locally computed hash an upstream provenance pin.
- Generate or fetch the verified stack/tokenizer/catalog and include them as resources. `bundle.active` becomes true only after sidecar/resource hashes and the actual package path pass. macOS nested-binary signing and a real application icon are addressed explicitly, not assumed from a 1×1 placeholder.
- The asset-origin CSP remains narrow. Host-mediated native/network actions do not require wildcard remote origins. Any verifier that previously asserted no native hooks or the old CSP is updated deliberately with its security rationale, not weakened silently.

## Verification and delivery

- Failing-first tests cover the canonical view map, one-state consent, exact program allowlist, bounded Git/terminal output, host sidecar proxy/lifecycle, remote JSON/SSE MCP, stdio MCP lifecycle, inert markup and package resource/hash validation.
- Existing Studio, Node, worker/page/WASM, native loopback, vision, xtask and desktop controls remain green except deliberately updated native-hook/CSP/theme contracts, which receive focused replacements.
- Real browser checks cover every view/submenu at desktop and mobile sizes. Real native GUI checks cover streaming, folder choice/file save, Git diff, terminal consent, local conversation restore and both MCP transports using fixtures. The packaged `.app` is launched, not merely compiled.
- Independent architecture and exact-head reviews precede protected merge. Fresh main, Pages runtime identity, public sidecar hashes and final free space are verified. No binary, corpus, secret or development-only file enters source or Pages commits.

## Explicit limits and follow-up

The first package target is macOS arm64; other platforms are not claimed. MCP support is bounded to initialize, tools list/call and declared transports, not every MCP feature or arbitrary server. Advanced agent autonomy, automatic Git writes, signed updater distribution, exact sidecar hidden-state resume and production model quality remain follow-up work.
