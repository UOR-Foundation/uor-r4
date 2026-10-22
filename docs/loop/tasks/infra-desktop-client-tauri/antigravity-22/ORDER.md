# ORDER: infra-desktop-client-tauri, antigravity-22

Lab: antigravity. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md). Issue: #2826. Base: `0aac25aca`.

## Goal

The item's done-when reads:

> A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. Until a trained model exists, "runs the engine" means the synthetic stand-in.

## Cancel probe (run first)

Ran `ls -la src-tauri` and `grep -n "https\?://" web/studio/index.html` at `0aac25aca`:
- `src-tauri/` does not exist; `web/studio/index.html` invokes `native_read_file`, `native_write_file`, `native_git_status`, `native_run_terminal_command`, and `native_list_local_models` with no Rust backend definitions.
- `web/studio/index.html` still loads `marked`, `katex`, `pdf.js`, `three.js`, `monaco`, and Google Fonts from external CDNs and leaves `callGithubApi` unguarded for offline desktop use.
Work is needed.

## Files in scope

- `src-tauri/**`
- `web/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-object-store/**`
- `docs/products/**`
- `Cargo.toml`
- `docs/labs/infra-desktop-client-tauri/**`

## Done when

1. Design note posted on `#2826` covering workspace placement, command types, granted-folder capability allowlist, library linkage, SHA-256 pin check before `format.rs`, terminal opt-in + consent + granted-folder guard, and zero softmax / zero matmul compliance.
2. `src-tauri/` crate builds and passes host tests (`cargo test --manifest-path src-tauri/Cargo.toml --locked`) verifying:
   - `native_list_local_models` discovers pinned `UORLUT01` files (including the seed-5 `synthetic-stack.lut` stand-in) and refuses unpinned or tampered files before `IntegerStackModel::parse`.
   - Native integer stack execution (`native_load_model`, `native_step_tokens`, `native_stream_turn`) produces the exact 32-token trajectory `[38, 9, 33, 8, 25, 38, 30, 25, 17, 30, ...]` from token `0` with `weights_per_token = 15968`.
   - Durable memory (`native_save_session_durable`, `native_restore_session_durable`) folds `UORMEM01` + `UR4SESS1` into `uor-r4-object-store::Store` (`DurableObjectMemoryStore`) and resumes token-for-token across close and restart (`16 + 16 == 32`).
   - Scoped file commands (`native_read_file`, `native_write_file`, `native_git_status`, `native_list_directory`) refuse paths outside granted folders or containing `..` traversal, and `native_run_terminal_command` is off by default and requires explicit per-command user confirmation inside a granted folder.
3. `web/studio/index.html` self-hosts its UI dependencies via `web/studio/assets/js/uor_offline_vendor.js` and `web/studio/assets/css/uor_offline_fonts.css`, routes Tauri desktop model/session calls through `window.__TAURI__.core.invoke`, and passes offline verification with zero external network calls.
4. `docs/products/desktop-client.md`, `docs/products/README.md`, and `docs/labs/infra-desktop-client-tauri/README.md` (with the `m4-serving` and `m3-durable-memory` notebook rows) are updated and all loop gates pass.

## Stop and ask if

- Any file outside the declared scope is required.
- Any GPU run or milestone bar change is needed.
