# REPORT: unified Apple-like web and Tauri harness IDE

Lab: kimi. Item: `infra-desktop-client-tauri` (#2826). Delivery scope: development verification only; the item is not complete.

## Negatives and limits first

- **Native GUI acceptance is NOT_RUN.** KimiCU was installed, but macOS reports `accessibility=false` and `screenRecording=false` after the permission request. No claim is made that native dialogs, chat, folder save, Git/terminal controls, or MCP were exercised through the real window.
- The first package omitted the `gui` feature and exited on stdin EOF; the acceptance build now uses `cargo tauri build --bundles app -- --features gui`.
- The first packaged launch selected development sidecar inputs when the worktree existed. A failing-first resolver regression now requires `.app` resources to win. The verified child uses only `UOR Harness.app/Contents/...` paths.
- Tauri initially placed resources under `Resources/runtime/studio`. An explicit source-to-destination map now produces `Resources/pkg` and `Resources/models.json`.
- The live browser exposed MCP removal during model-source rerenders. MCP now has an independent `#mcpMount` sibling and a rerender regression.
- One remote MCP fixture run failed with a response-body error. Its isolated rerun and the following complete desktop suite passed; this is retained as a fixture-timing observation, not hidden as a mechanism negative.
- No useful conversation, visual understanding, exact sidecar hidden-state resume, signed distribution, production readiness, M3/M7/M8 completion, or single-binary app is claimed.

## Measured implementation

- One staged frontend and backend state; one authenticated `uor-chat` sidecar reached only through host-mediated Tauri IPC.
- Live SSE chat, model listing/testing and exact image decomposition; cancellation and bounded event queues.
- Native project/model-folder grants on shared state, file read/write, NUL-safe bounded Git status/diff, host-confirmed exact-program terminal execution with timeout/output caps.
- Browser remote MCP and native remote/stdio MCP with bounded initialize/list/call, JSON/SSE normalization, memory-only credentials, real loopback HTTP and compiled stdio fixtures.
- Runtime-only allowlist shared by Tauri and Pages. No development source, tests, package configuration, declarations, verifier files, README or `node_modules` in the package.

## Package evidence

Build command:

```sh
PATH=/Users/casey.allard/.cargo/bin:$PATH \
CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/kimi/harness-app-target \
  cargo tauri build --bundles app -- --features gui
```

Bundle: `/Users/casey.allard/uor-r4/local/kimi/harness-app-target/release/bundle/macos/UOR Harness.app`.

- `python3 scripts/tests/test_tauri_app.py` with `UOR_TAURI_APP` set: **2/2 PASS**.
- Sidecar SHA-256: `f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e`.
- Stack SHA-256: `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`.
- Tokenizer SHA-256: `6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b`.
- Catalog SHA-256: `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.
- Desktop executable SHA-256: `d15857a75667d4e7c356d41fa7b399c4d8c5c2a10dc20976cc2026ade79b1d48`.
- Real launch produced `uor-desktop-client` and `uor-chat` from the `.app`; arguments used packaged stack/tokenizer and `--token-stdin`, with no token in argv. Both were stopped and no child remained.
- This is an unsigned development bundle, not a signed release.

## Checks

- Studio: **298/298** Vitest tests and TypeScript typecheck.
- JavaScript/Node: **58/58** tests, including matched JSON/SSE MCP and native host selection.
- Desktop GUI-feature suite: **11** controls, **4** MCP protocol fixtures, **4** sidecar tests and **2** library tests passed.
- Runtime preparation, resource hash verification and package acceptance pass.
- Runtime page-load trap: no external requests; desktop and 390px mobile Connections/MCP views inspected in the built-in browser.
- Exact native GUI acceptance remains **NOT_RUN** for the permission reason above.

## Remaining work

Enable macOS Accessibility and Screen Recording for KimiCU, then exercise every native acceptance row with real GUI interaction and fixtures. Complete independent review, protected delivery, fresh-main and hosted verification, and final cleanup with at least 30 GiB free. #2826 remains open.
