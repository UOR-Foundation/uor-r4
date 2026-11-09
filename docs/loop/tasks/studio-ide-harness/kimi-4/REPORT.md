# REPORT: replace one-shot Terminal with an interactive PTY, kimi-4

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `f86f4c1165d71cd553dd179d8f37db5cfc19b619`. This is unsigned development-terminal evidence, not a hosted terminal service or full Cowork/agent claim.

## What did not work

- The old panel issued one approved command per Run button. It was not a terminal. A plain `cargo build` can also embed an older `runtime/studio`; native inspection found that old panel until the preparation script refreshed the staged frontend.
- Live acceptance found Rust emitting `data_hex`/`session_id` while TypeScript expected `dataHex`/`sessionId`. A JSON regression failed with the snake-case event before the fix. No dual-name fallback was added.
- The merged full compile found `DesktopBackend` was not `Sync` because of raw PTY master/writer handles. The new `Send + Sync` regression failed before those handles were protected with mutexes. The interactive tests now pass alongside existing scoped-thread host tests.
- The new stale-open regression failed: a shell opening after a workspace change could mount and poll in the new project. It now closes that stale shell. `native_open_terminal` also failed the async-dialog regression because it was synchronous despite showing approval UI; it now follows the other async native commands, with exactly five main-window confirmation sites.
- A proposed process-control watchdog regressed live-child and descendant termination and was reverted completely. The inherited process runner and its original bounds are preserved. Earlier concurrent visual/MCP timeout failures are not hidden; final verification is serial.
- Initial acceptance scripts had invalid selectors/syntax, closed the target tab, raced folder loading, mismatched approved/default shells, omitted page sessions, and sent an extra newline after `read`. Those harness failures are not claimed as product results.
- A post-acceptance source review found that shell approval remained in a set after opening, so one grant could open multiple shells. `pty_shell_approval_is_consumed_per_open` reproduced that authorization defect before the check was changed to consume the grant. The old bounds test reused one approval for eight sessions; it now approves each intended shell and asserts the ninth refusal is specifically `SessionLimitReached`.

## What changed

- `portable-pty = "0.9.0"` owns persistent child/master/writer/reader state, monotonically sequenced raw-byte events, and explicit shutdown. At most eight sessions, 64 KiB per input, and 1 MiB retained output per session are enforced. Only `/bin/zsh`, `/bin/sh`, and `/bin/bash` are accepted under approved canonical working directories.
- `@xterm/xterm = "6.0.0"` and `@xterm/addon-fit = "0.11.0"` render the terminal with direct `onData` input, resize, 50 ms read-cursor polling, and workspace/unmount/close cleanup. There is no Run command button or per-keystroke approval; raw bytes never become HTML.
- `native_open_terminal` is async, approval wording describes per-shell/one-shot consent, and stale opens close rather than mount. The one-shot API remains separate for compatibility. `verify_studio_page_load.mjs` has an opt-in `keepOpen` test flag; its default close behavior is unchanged.
- The predecessor's MCP test-only fixture stabilization is reused from `af86d7ca0`, without importing its ORDER or production changes. No model/API/engine/worker/tokenizer implementation was edited.

## Measurements

- Earlier default Rust: **11/2/20/3/7/13/8** and GUI Rust: **13/2/24/8/13/9**, with interactive **3/3** in each mode. After the shell-approval-consumption repair, default Rust is **11/2/20/4/7/13/8** (library/main/controls/interactive/MCP/review/sidecar) and GUI Rust is **13/2/24/4/8/13/9**, PASS, including actual PTY input/read reply/cwd/resize/close, one-shell-per-approval refusal and renewal, distinct session-cap refusal, JSON, and `Send + Sync` contracts.
- Frontend: **442/442** Vitest across **52 files**, **113/113** plain-JS tests, **45/45** Playwright checks, TypeScript and Vite build PASS, rerun serially. Stale-open component regression: **RED → GREEN 10/10**.
- Earlier real isolated loopback `/bin/zsh`: `READY`, `ANSWER:fixture-answer`, granted `pwd`, `stty size` = **24 133**, exit 0, and later-input refusal. Initial requests **15**, external/raw-TypeScript/runtime errors **0/0/0**. Evidence: `local/kimi/harness-parity/fe5fa9115b06f682cf382a31d7fabc543d6c1a48/interactive-terminal.{json,png}`; pre-commit host `286e1d7619abc3fefc4d9b6c399df756f7273e6c6506ec2aa8e1d84d27a1cd14`, dist `ec1dd0d807add48f05e89c484f98d2dfb9643e43dd0a7c7b764da1436bfbd98e`. It is not represented as final-head evidence.
- Earlier unsigned native GUI at the same host identity: `NATIVE_READY`, exact isolated `pwd`, and `Close shell` reaped PID `18600` (`ps` returned no row). Screenshot `local/kimi/harness-parity/native-interactive-terminal.jpg`, SHA-256 `a613f296596fbfb6d88bcd0ddc8deac9fb9b7edbe99524376d1d493d95c989bd`. Final current-head host/evidence is below.

- Final-release native acceptance at `975ebff87657106914f8078c5a5e7dc75ad00329`, tree `1e246dd3f3233331503b6cd051f22db899179409`, used the actual `org.uor.r4.desktop.cowork` bundle with only an isolated fixture grant and a single pre-approved `/bin/zsh`. The real native enablement sheet was then approved for that fixture. `NATIVE_READY`, the exact fixture `pwd`, a subsequent `read` answer `ANSWER:fixture-answer`, and `stty size` = **24 80** were observed. Close shell returned Open terminal and `ps -p 77421` returned no row for the previously observed shell child. Executable SHA-256 `3609fc9846d6ba420dfd06807cb1b3902235620aaf70a096c699423fe32d38a7`; screenshot SHA-256 `db1cce2b9eb16879892b3714226837cdbe94650d602c625dc12b0ca57a69ecda`. Full observation and limits: `local/kimi/harness-parity/975ebff87657106914f8078c5a5e7dc75ad00329/native-terminal.md`. No detached-descendant containment claim follows.
- Dependencies are exactly the three declared MIT packages: `portable-pty`, `@xterm/xterm`, and `@xterm/addon-fit`, with lockfile updates. No ttyd, CDN, public socket, or other dependency was added.

## Checks run at the head

- `CARGO_TARGET_DIR="$PWD/target/cowork-tests" cargo test --manifest-path src-tauri/Cargo.toml`: complete default PASS; the same with `--features gui`: complete GUI PASS.
- `npm test --prefix web/studio`: **442/442**. `npm run typecheck` and `npm run build`: PASS. `node --test web/studio/assets/js/*.test.mjs`: **113/113**. `npm run test:e2e --prefix web/studio`: **45/45**.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check`: PASS.
- Current committed-head loopback, repository gates, fresh exact-head review and protected delivery are recorded only after they execute.

## Skills used

Systematic root-cause investigation, red-first PTY/JSON/concurrency/stale-open/async regressions, native macOS accessibility inspection, isolated Chrome/Playwright acceptance, serial full-suite verification, protected delivery preparation, and verification before completion.

## Still open

Current-head acceptance, repository gates, fresh adversarial review, PR/GitHub checks, protected merge and fresh-main verification. The shared persistent agent runtime, parallel projects/tasks, memory, scheduling/Dispatch, browser/computer workflows, and Claude-owned R4 API adapter remain separate work. No public ttyd endpoint, detached-process-containment guarantee, production publication, signed release, useful-model result, or full Cowork parity is established.

## Plain summary

The neutral app now has a real persistent PTY with direct keyboard input and streamed output, not a command button. Native and loopback shells have returned real output and closed cleanly. The remaining work is protected delivery and the shared agent/runtime functionality needed for full Cowork capability.