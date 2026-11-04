# REPORT: unified Apple-like web and Tauri harness IDE

Lab: kimi. Item: `infra-desktop-client-tauri` (#2826). Delivery scope: development verification only; the expanded harness and hosted update are not yet delivered.

## Negatives and limits first

- The sidecar serves the synthetic stack, not useful conversation or visual understanding. No production-readiness, M3/M7/M8-completion, exact sidecar hidden-state resume, signed-distribution or single-binary claim is made.
- Normal app exit killed the app, `uor-chat` and stdio fixture. An earlier forced SIGTERM left an orphaned sidecar, which was killed by exact PID; forced-kill cleanup is not claimed. The ORDER requires normal exit only.
- Cmd+R did not reload the Tauri page or end the stdio child, so it was not evidence about credential clearing. A real close/restart left the credential field empty and MCP disconnected, while the saved conversation remained.
- Model/file/durable-store operations can still serialize on the backend mutex. Blocking Tauri commands run on workers, keeping the main UI thread free; parallel storage access is not claimed. The terminal is an explicitly approved tool runner, not a general process or filesystem sandbox.
- The initial packaged native transport accepted origin aliases/non-default ports, dropped delayed/partial responses, panicked on async remote-MCP errors, mishandled cancellation, allowed attached-option and Git-hook escapes, and had unsafe durable-restore paths. Failing-first regressions drove fixes for all of these. The first test-side response fixture also needed to consume the request body; that was an instrument correction, not a model result.
- Exact-head independent review, protected merge, fresh-main verification, hosted Pages publication and the 30 GiB free-space target remain unfinished at this report's current state.

## Measured implementation

- One staged frontend/backend state and one Tauri asset origin, with exact scheme/host/default-port checks and no userinfo. A host proxy reaches the pinned `uor-chat` sidecar through Tauri IPC; the page never receives its token or contacts it directly.
- Persistent byte buffers and 15-second idle deadlines preserve delayed/split JSON and SSE byte-for-byte. Blocking native commands run off the main thread; remote and stdio MCP connection work uses blocking workers, with errors rather than async-runtime panics.
- Chat, sessions/history, long paste/attachments, native model list/test and exact image decomposition use the shared OpenAI transport. Request IDs are page-unique; cancellation and bounded event queues are explicit.
- Native project/model-folder grants share file, bounded Git and terminal state. Terminal enablement and each exact command use native approval, with a 60-second one-shot grant, direct allowlisted execution, metacharacter/attached-file-option refusal, hardened Git, and bounded output/time.
- Durable manifests use collision-resistant scope filenames, same-directory temporary files and atomic rename; restore rejects `scopes` symlinks and verifies the granted manifest path. This older engine path is not represented as sidecar hidden-state resume.
- Browser remote MCP and native remote/stdio MCP implement bounded initialize/list/call, JSON/SSE normalization and explicit approval. Credentials remain memory-only. Runtime-only allowlists stage the same files for Tauri and Pages, excluding development sources/tests/configuration/declarations/verifiers/README/`node_modules`.

## Real native GUI evidence

The final formatted app was exercised through the actual macOS window and native sheets:

- Existing `Hi` history displayed 16 completion tokens, exact usage `19/16/35`, `14985.9 tok/s`, TTFT `0.459 ms`, total `1.068 ms`. This is the earlier synthetic timing record, not a new quality benchmark.
- `note.txt` retained the exact saved text `Native GUI note\nSaved through native host\n`; Changes previously showed `main · 1 changed file` and the exact `+Saved through native host` diff. `pwd` previously returned the disposable project path through both native terminal sheets.
- Remote MCP at `127.0.0.1:9000/mcp` showed `Connected · 1 tool`; the parented approval sheet explicitly said `A credential will be sent to this server.` `read_file` returned `native remote result`.
- The native folder picker granted only the disposable fixture directory. Stdio approval showed the exact executable, empty args and working directory; `read_file` returned `stdio result`.
- Native Cancel displayed `Connection approval was cancelled; nothing started.` Normal Cmd+Q left no app, sidecar or stdio child (checked by exact PID). A real restart showed an empty credential field and no MCP session, with saved chat history intact.

## Package evidence

Prepare/build commands:

```sh
./scripts/studio/prepare-tauri-runtime.sh
PATH=/Users/casey.allard/.cargo/bin:$PATH \
CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/kimi/harness-app-target \
  cargo tauri build --bundles app -- --features gui
```

Bundle: `/Users/casey.allard/uor-r4/local/kimi/harness-app-target/release/bundle/macos/UOR Harness.app` (unsigned development package).

- `UOR_TAURI_APP=... UOR_DESKTOP_SHA256=<fresh executable hash> python3 scripts/tests/test_tauri_app.py`: **2/2 PASS**.
- Desktop executable SHA-256: `20cc8347c3346cb59600d4557ab0f8a48e860d4df24b6deb24baef4930893f9f`.
- Sidecar SHA-256: `f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e`.
- Stack SHA-256: `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`.
- Tokenizer SHA-256: `6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b`.
- Catalog SHA-256: `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.

## Automated checks

- Studio TypeScript typecheck and **300/300** Vitest tests.
- JavaScript/Node **65/65** (the earlier incorrect working-directory glob reported zero tests and was discarded, not counted).
- Default desktop: **2** library, **12** controls, **4** MCP, **5** review and **7** sidecar tests passed.
- GUI-feature desktop: **4** library, **16** controls, **5** MCP, **5** review and **8** sidecar tests passed.
- Runtime preparation, resource identity and packaged app verification passed. No GPU or paid compute was used.

## Remaining work

Run final formatted-build tests and package verification, commit named paths, merge current `origin/main`, run loop checks and fresh-context exact-head adversarial review, deliver through the protected PR/queue, verify fresh `main`, publish only the runtime allowlist to Pages, and clean regenerable local files without deleting evidence or other labs' work. #2826 remains open; use `References #2826`, not `Closes`.
