# REPORT: isolate MCP and process timeout fixtures, kimi-77

Lab: kimi. Item: studio-typed-shell-upgrade (#3500), predecessor PR #3879. Order: [ORDER.md](ORDER.md). No production behavior changed.

## Negatives first

- The complete default suite at `759da8585` passed library 9, CLI 3 and controls 21, then failed the remote MCP timeout/size test with a short-timeout transport error and a broken-pipe fixture write. Subsequent full runs exposed unexpected 400/401 classifications and descendant PID-file startup races.
- A focused MCP run initially passed after delaying only `tools/call`; that was not accepted as proof while the full suite exposed incomplete accepted-socket reads. Accepted fixture sockets now explicitly block under their one-second bound, and partial frames are refused before credential evaluation.
- Two 25 ms descendant fixtures could expire before creating the PID file. The fixtures now synchronize readiness through a pipe and use a one-second startup-plus-timeout bound. The timeout and no-surviving-descendant assertions remain intact.
- The fresh loopback evidence names the still-uncommitted fixture sources in its source status; it is not relabeled as clean-head evidence. Final repository gates and exact-head review remain pending.

## Measurements

- Complete default Rust suites: **9/3/21/7/13/8** (library/main/controls/MCP/review/sidecar), PASS.
- Complete GUI Rust suites: **11/3/25/8/13/9**, PASS.
- The remote timeout test completes ordinary initialize/notification before its original 15 ms hanging tool call; the 1024-byte response cap still fails explicitly. Stdio retains its 25 ms hanging-call assertion and cleanup checks.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check`: PASS after formatting.
- Predecessor typed/plain-JS/runtime and fresh identity-bound loopback results are in `../kimi-75/REPORT.md`.

## Scope and remaining delivery

Only `src-tauri/tests/mcp_protocol.rs`, test code in `src-tauri/src/lib.rs`, and this record/ORDER were changed by this repair. Production MCP/process code, R4 API/model/engine/worker/tokenizer and application visuals were not changed. Full repository gates, exact-head review, PR checks and protected delivery remain separate pending steps.
