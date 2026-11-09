# REPORT: bind the final Studio delivery evidence and records, kimi-79

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Review base: `3c7176d5f37c2b98329877e6c6388fb63ed8b88e`; merge base `0895aa65ff2f63ed184e78e81c28e67ffa3f34aa`. Fresh reviewer agent-62 returned CHANGES; #3879 is not queued.

## What did not work

- The earlier report treated `d39953a7d` as the final head without naming the actual pushed record head, its tree, or the merge base. The full-gate result was contradicted by “final gate still required” wording, and local-policy/default/GUI outputs were asserted without retained logs.
- Active desktop/current-state/notebook/README records still pointed to older counts and the kimi-71 report. Gate evidence existed only in the worktree, so deleting it would lose unique evidence.
- The MCP assertion checked a tool name and credential but not the complete JSON-RPC body, empty arguments, or protocol/session headers. This is a fixture-evidence gap, not a production protocol change.

## What changed

- The MCP regression now asserts the complete authorized body: `jsonrpc = 2.0`, `id = 3`, `method = tools/call`, `params.name = read_file`, and `params.arguments = {}`. Authorization, content type, `Mcp-Session-Id`, and `MCP-Protocol-Version` are asserted. Expected disconnects are tolerated; unexpected fixture-thread panics fail. Original 15 ms/25 ms limits remain.
- Default-root coverage includes all lowercase/uppercase/mixed-case source and map variants. Desktop controls are now documented as **25 GUI / 21 default**, with current `kimi-78/79` evidence. Current-state and notebook rows describe the repair and preserve historical GUI scope.
- `d39953a7d4ebab479673eba50e06481151b740bc` is the earlier source/acceptance head, not the final record head. `d39953a7` through `3c7176d5f` was record-only. Source/acceptance head `988155ae56dc0fdd872b7428018ead566179fe3f` has tree `e6542fb375518c1b38639cff825b9ac65f7b3594`; subsequent REPORT/evidence-only deltas do not change production source.
- Full default/GUI logs are retained under `local/kimi/studio-final-acceptance/988155ae56dc0fdd872b7428018ead566179fe3f/{default-tests,gui-tests}.log`. The d399 gate summary and runner directory are copied byte-identically to the main checkout at `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/d39953a7d4ebab479673eba50e06481151b740bc/{full-gates.log,full-gate-runner/}`; the summary SHA-256 is `723ba3ee1e110db412525e8ce78a263512a33a9fda0b7c398e9b595416c44c17`.

## Measurements

- Full default suites: **11/3/21/7/13/8**, PASS; complete GUI suites: **13/3/25/8/13/9**, PASS. Full MCP fixture: authorized complete request/envelope/header assertion, original hanging-call limits, expected disconnect handling, and unexpected-panic detection.
- Mixed-case source request: **RED `/src/main.TS` = 200 → GREEN 404**. Valid canonical roots still work, regular-file explicit roots fail, and valid assets still serve.
- d399 clean loopback: 15 initial/12 reopened requests, exact transcript/session identity and native edit/save, zero external/raw-TypeScript/runtime errors. Host SHA-256 `430ac620f81ac73e67f9bf4aa63f70bff4f9f9add8285f24f390d1d7f92e9dd4`; dist SHA-256 `30358639f919fc55fb8fa75b6daf6df71e33e10ac9da7cb5b02452ce51fade55`.
- d399 full repository gates: **11/11 PASS** at `d39953a7d4ebab479673eba50e06481151b740bc`, with durable summary/runner evidence above. This is not substituted for final-head policy/gate evidence.

## Checks run at the head

- `CARGO_TARGET_DIR="$PWD/target/acceptance-tests" cargo test --manifest-path src-tauri/Cargo.toml`: complete default PASS, retained in `default-tests.log`.
- The same with `--features gui`: complete GUI PASS, retained in `gui-tests.log`.
- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_client_controls every_static_root_refuses_source_typescript_and_source_maps -- --exact --nocapture`: PASS after the demonstrated mixed-case failure.
- `cargo test --manifest-path src-tauri/Cargo.toml --test mcp_protocol remote_auth_timeout_and_size_caps_fail_explicitly -- --exact --nocapture`: PASS with the complete request/envelope/header assertion.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check`: PASS. Final record-only policy, full-gate, loopback and independent review are recorded after they execute.

## Skills used

Fresh exact-head adversarial review, red-first mixed-case and complete-MCP-envelope regressions, source/tree/merge-base binding, full default/GUI evidence preservation, durable artifact copying, current-state/notebook/README synchronization, and verification before completion.

## Still open

Final record-only `pr-check --local`, full repository gates and loopback with retained output; fresh exact-head approval; required GitHub checks; protected merge; and fresh-main verification. No production publication, signed release, full Cowork parity, useful-chat result, or fresh packaged-GUI acceptance is claimed.

## Plain summary

The code-level mixed-case source boundary is fixed. The remaining work is proof of delivery: the final head and tree are named, fixture assertions cover the complete request, and suite/policy/gate evidence is retained in the main checkout rather than only the worktree.