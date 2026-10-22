# Lab Record: Tauri Desktop Client, Native Integer Serving, and Durable Object-Store Sessions (`#2826`)

> **Claim discipline:** 0/8 milestones are complete; no capability is established. All native serving and durable memory verification in this record uses the 469,252-byte `mk-synthetic-stack` (`seed = 5`, `UORLUT01`) synthetic stand-in artifact (`sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`). No trained model exists (`M1`–`M4` `0/8`), and no native vs. WebAssembly decode speed is claimed (`M5` `0/8`).

## 1. Summary

This record documents the implementation of the Tauri desktop client (`src-tauri/`, Issue `#2826`, [`docs/roadmap/items/infra-desktop-client-tauri.md`](../../roadmap/items/infra-desktop-client-tauri.md)) and the offline Studio desktop bridge ([`web/studio/index.html`](../../../web/studio/index.html), [`web/studio/assets/js/uor_offline_vendor.js`](../../../web/studio/assets/js/uor_offline_vendor.js)):

1. **Workspace placement:** [`src-tauri/Cargo.toml`](../../../src-tauri/Cargo.toml) is a standalone Cargo package listed under `exclude = [..., "src-tauri"]` in the root [`Cargo.toml`](../../../Cargo.toml) so `cargo test --workspace` stays free of system GUI/WebView dependencies while `cargo test --manifest-path src-tauri/Cargo.toml --locked` compiles and tests the native desktop backend directly.
2. **Native D11 integer serving (`uor-r4-integer`):** [`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs) links [`crates/uor-r4-integer`](../../../crates/uor-r4-integer/) (`LoadedLutStack`, `LutStackSession`, `mk_synthetic_stack_bytes`) with `#![forbid(unsafe_code)]`, zero softmax/squashing (D23), and zero matmul (D25). `native_load_model` verifies `b"UORLUT01"` magic and checks SHA-256 against the expected digest before invoking `LoadedLutStack::from_bytes`.
3. **Durable session memory (`uor-r4-object-store`):** `native_save_session_durable` and `native_restore_session_durable` wrap `UR4SESS1` session snapshots (`LutStackSession::to_session_bytes` / `from_session_bytes`) inside a `UORMEM01` envelope (`encode_memory_envelope`, `fold_memory_envelope`, `reconstruct_memory_envelope`, `verify_memory_envelope`) persisted in `uor-r4-object-store::Store` (`UOROBJ01`).
4. **Scoped file, Git, and terminal controls:** `GrantedFolders` canonicalizes user-granted workspace roots and refuses `native_read_file`, `native_write_file`, `native_list_directory`, and `native_git_status` outside granted roots (including `..` traversal). `native_run_terminal_command` is disabled by default (`terminal_enabled = false`), requires explicit per-command confirmation (`user_confirmed: true` with `confirmed_command == command`), and executes via `sh -c` only inside a granted root.
5. **100% offline Studio frontend:** [`web/studio/index.html`](../../../web/studio/index.html) replaces all external CDN `<script>` and `<link>` tags (`marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) with [`web/studio/assets/css/uor_offline_fonts.css`](../../../web/studio/assets/css/uor_offline_fonts.css) and [`web/studio/assets/js/uor_offline_vendor.js`](../../../web/studio/assets/js/uor_offline_vendor.js) (`0` external `http://` or `https://` URLs), and [`src-tauri/tauri.conf.json`](../../../src-tauri/tauri.conf.json) restricts `connect-src` to `'self' ipc: http://ipc.localhost`.

## 2. Verification Receipts

### 2.1 Native Rust Integration Controls (`src-tauri/tests/desktop_client_controls.rs`)

Command:

```sh
CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked
```

All `5/5` integration controls pass:

| Test | Verified contract | Result |
|---|---|---|
| `native_list_local_models_returns_synthetic_standin_and_refuses_tampered_hash` | Discovers `UORLUT01` (`469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`), refuses non-`UORLUT01` magic, and refuses mismatched SHA-256 before `format.rs` | `ok` |
| `native_engine_step_tokens_matches_reference_32_token_trajectory` | 32-token greedy trajectory from `PROMPT = [0]` matches `docs/wasm-serving/verify.mjs` (`[38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30]`), `weights_per_token = 15,968` | `ok` |
| `durable_memory_object_store_close_and_resume_continues_token_for_token` | Steps 16 tokens (`UR4SESS1` `5,366 B`), folds `UORMEM01` into `uor-r4-object-store::Store` on disk, drops `DesktopState`, reconstructs and verifies `UORMEM01` + `UR4SESS1` in a fresh `DesktopState`, and steps 16 more tokens; combined `16 + 16` matches the unsplit 32-token trajectory token-for-token | `ok` |
| `scoped_file_and_git_commands_enforce_granted_folder_allowlist` | Refuses `native_read_file`, `native_write_file`, `native_list_directory`, and `native_git_status` outside `GrantedFolders` (including `../` traversal); succeeds after `native_grant_folder` | `ok` |
| `terminal_command_is_off_by_default_and_requires_consent_and_granted_folder` | Refuses `native_run_terminal_command` when `terminal_enabled == false`, when `user_confirmed == false`, when `confirmed_command != command`, or when `cwd` is outside `GrantedFolders`; succeeds only when all four guards hold | `ok` |

### 2.2 Native CLI Self-Test (`uor-desktop-client --self-test`)

Command:

```sh
CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo run --manifest-path src-tauri/Cargo.toml --locked -- --self-test
```

Output:

```json
{"artifactBytes":469252,"artifactSha256":"d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df","durableScope":"local-user/studio/self-test","engine":"uor-r4-integer/native-desktop","objectVersion":"bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca","storeRecords":446,"streamedCount":32,"tokens32":[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"ur4sess1Bytes":9782,"verdict":"PASS","weightsPerToken":15968}
```

### 2.3 Offline Studio & Tauri Bridge Verification (`web/studio/verify_desktop_client.mjs`)

Command:

```sh
UOR_DESKTOP_BIN=./local/tauri-target/debug/uor-desktop-client node web/studio/verify_desktop_client.mjs
```

Output:

```json
{"verdict":"ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED","offlineExternalUrls":0,"invokedCommands":["native_load_model","native_step_tokens","native_save_session_durable"],"selfTestReceipt":{"artifactBytes":469252,"artifactSha256":"d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df","durableScope":"local-user/studio/self-test","engine":"uor-r4-integer/native-desktop","objectVersion":"bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca","storeRecords":446,"streamedCount":32,"tokens32":[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"ur4sess1Bytes":9782,"verdict":"PASS","weightsPerToken":15968},"remoteRequests":0}
```

## 3. Six-Column Research Notebook Rows

Because `docs/research/notebook/` is outside the `scope:` declaration on [`docs/roadmap/items/infra-desktop-client-tauri.md`](../../roadmap/items/infra-desktop-client-tauri.md), the two six-column research notebook rows for `m4-serving.md` and `m3-durable-memory.md` are recorded here for verbatim inclusion when a notebook-scoped item lands:

### Row for `docs/research/notebook/m4-serving.md`

| Date | Decision / Issue | Change | Result | Classification | Artifact / Commit |
|---|---|---|---|---|---|
| 2026-10-22 | `#2826` (`infra-desktop-client-tauri`, M4 `#2032` / M7 `#2035`) | Implemented `src-tauri/` native desktop client linking `uor-r4-integer` (`LoadedLutStack`, `LutStackSession`) and self-hosted all Studio vendor assets (`uor_offline_vendor.js`, `uor_offline_fonts.css`) with zero external URLs | `5/5` integration controls pass; 32-token synthetic stand-in (`seed = 5`, `469,252 B`, `sha256:d60f88d6...`) matches `docs/wasm-serving/verify.mjs` token-for-token (`[38,9,33,8,25,38,30,25,17,30,30,30,17,30...]`, `15,968 B` weights/tok); `0` remote requests | **Proof / finite host check** (synthetic stand-in only; no trained model or decode speed claim) | `src-tauri/src/lib.rs`, `web/studio/verify_desktop_client.mjs`, [`docs/labs/infra-desktop-client-tauri/README.md`](./README.md) |

### Row for `docs/research/notebook/m3-durable-memory.md`

| Date | Decision / Issue | Change | Result | Classification | Artifact / Commit |
|---|---|---|---|---|---|
| 2026-10-22 | `#2826` (`infra-desktop-client-tauri`, M3 `#2031` / D19) | Wired `native_save_session_durable` and `native_restore_session_durable` in `src-tauri/` to fold and verify `UORMEM01` envelopes containing `UR4SESS1` session snapshots in `uor-r4-object-store::Store` | 32-token synthetic session folds into `446` `UOROBJ01` records (`UR4SESS1` `9,782 B`, `object_version = bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca`); `16 + 16` close-and-resume across fresh `DesktopState` instances matches the unsplit 32-token trajectory token-for-token | **Proof / finite host check** (synthetic stand-in only; M3 `0/8` until tested on a trained model) | `src-tauri/src/lib.rs`, `src-tauri/tests/desktop_client_controls.rs`, [`docs/labs/infra-desktop-client-tauri/README.md`](./README.md) |
