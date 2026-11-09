# REPORT: replace one-shot Terminal with an interactive PTY, kimi-4

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `f86f4c1165d71cd553dd179d8f37db5cfc19b619`. This is unsigned development-terminal evidence, not a hosted terminal service or full Cowork/agent claim.

## Negatives first

- The old panel issued one approved command per Run button. It was not a terminal.
- The normal app can open a stale embedded frontend if only `cargo build` is rerun. Actual native inspection found the old one-shot panel while the current browser UI had the PTY implementation. `scripts/studio/prepare-tauri-runtime.sh` and the canonical launcher must refresh the staged assets before native acceptance.
- Initial acceptance scripts had invalid syntax/selectors, closed the target tab, raced the folder picker, approved `/bin/sh` while the UI requested `/bin/zsh`, used browser-level input without a page session, and supplied an extra newline after `read`. These harness failures are not hidden as application results.
- Live browser acceptance found a real contract bug: Rust emitted `data_hex`/`session_id` while TypeScript expected `dataHex`/`sessionId`. A JSON serialization regression failed with `{"data_hex":...,"session_id":...}` before the fix and now passes. The frontend does not silently accept both names.
- Parallel full-suite runs produced timeout-only failures in the old visual and MCP tests. Serial verification is required for this worktree; the failed concurrent results are retained, not described as product failures or passes.
- The terminal remains local/native-only. Static browser has no session service; no public ttyd endpoint, hosted production terminal, arbitrary shell grant, detached-process-containment guarantee, signed release, useful-model result, or full agent runtime is established.

## Implementation

- `portable-pty = "0.9.0"` owns persistent child/master/writer/reader state, monotonically sequenced raw-byte events, and explicit shutdown. At most eight sessions, 64 KiB per input, and 1 MiB retained output per session are enforced. Only `/bin/zsh`, `/bin/sh`, and `/bin/bash` are accepted under approved canonical working directories.
- `@xterm/xterm = "6.0.0"` and `@xterm/addon-fit = "0.11.0"` render the persistent terminal with direct `onData` input, resize, 50 ms read-cursor polling, and workspace/unmount/close cleanup. There is no Run command button and no per-keystroke approval. Raw bytes never become HTML.
- The existing one-shot API remains for compatibility. PTY commands use `native_open_terminal`, `native_read_terminal`, `native_terminal_input`, `native_resize_terminal`, and `native_close_terminal`; CLI `--approve-shell <cwd> <shell>` and isolated `--terminal-home <dir>` support explicit fixtures.
- `verify_studio_page_load.mjs` has an opt-in `keepOpen` test flag; its default close behavior is unchanged. The enablement dialog now describes per-shell/one-shot approval rather than incorrectly implying approval for every keystroke.

## Measured evidence

- Real isolated loopback `/bin/zsh`: `READY`, `ANSWER:fixture-answer`, granted `pwd`, `stty size` = **24 133**, exit 0, and later-input refusal. Initial requests: **15**, external/raw-TypeScript/runtime errors: **0/0/0**. Evidence: `local/kimi/harness-parity/fe5fa9115b06f682cf382a31d7fabc543d6c1a48/interactive-terminal.{json,png}`. Pre-commit host SHA-256: `286e1d7619abc3fefc4d9b6c399df756f7273e6c6506ec2aa8e1d84d27a1cd14`; dist combined SHA-256: `ec1dd0d807add48f05e89c484f98d2dfb9643e43dd0a7c7b764da1436bfbd98e`. Its source status explicitly records the then-uncommitted implementation; exact delivery-head evidence follows after the merge.
- Native unsigned debug GUI at the same host identity: the neutral Terminal view mounted xterm, `printf` returned `NATIVE_READY`, and `pwd` returned the exact isolated directory. `Close shell` reaped child PID `18600`; `ps` returned no row. Screenshot: `local/kimi/harness-parity/native-interactive-terminal.jpg`, SHA-256 `a613f296596fbfb6d88bcd0ddc8deac9fb9b7edbe99524376d1d493d95c989bd`. No real shell profile or external endpoint was used.
- Default and GUI interactive PTY tests: **2/2 each**, including actual PTY input/read reply/cwd/resize/close, approval/refusal, output/input/session bounds, and the JSON contract regression. Rust formatting is clean.
- Frontend: **441/441** Vitest across **52 files**, **113/113** plain-JS tests, **45/45** Playwright checks, TypeScript and Vite build PASS. These were rerun serially after the concurrent timeout failures.
- New runtime dependencies are exactly the three named MIT packages above, with lockfile updates; no ttyd, CDN, public socket, or other dependency was added.

## Still open

Merged-current-main full default/GUI suites, refreshed exact-head acceptance, repository gates, fresh adversarial review, PR/GitHub checks, protected merge and fresh-main verification. The shared persistent agent runtime, projects/task orchestration, browser/computer workflows, and Claude-owned R4 API adapter follow separately. No useful conversation or full Cowork parity is claimed.
