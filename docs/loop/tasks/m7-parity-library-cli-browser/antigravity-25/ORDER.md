# ORDER: m7-parity-library-cli-browser, antigravity-25

Lab: antigravity. Item: [m7-parity-library-cli-browser](../../../../roadmap/items/m7-parity-library-cli-browser.md). Base: `bca96e43d`.

## Goal

The item's done-when reads:

> M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. Today there is no accepted artifact, so a harness that passes on the synthetic stack (`mk-synthetic-stack`) shows the harness works and is not a milestone result. The bar is not yet registered for how many prompts, how many tokens per prompt or which sampler settings the comparison covers; registering it is step 2 below.

## Cancel probe (run first)

- Checked `crates/uor-r4-integer/tests/`, `crates/uor-r4-integer/src/bin/uor-chat.rs`, `web/studio/`, and `docs/labs/m7-parity-library-cli-browser/`:
  - `docs/labs/m7-parity-library-cli-browser/` does not exist yet (`0` comments on Issue `#2867`).
  - `uor-chat --stack` in `crates/uor-r4-integer/src/bin/uor-chat.rs` has no `--save-session <path>`, `--restore-session <path>`, or `--prompt-ids <csv>` flags to emit and reload canonical `UR4SESS1` session snapshots across fresh CLI processes.
  - No Rust test or Node/WASM harness compares token trajectories and `UR4SESS1` session snapshots across all three serving legs (Rust library `IntegerStackModel`/`IntegerStackSession`, `uor-chat` CLI, and Browser WASM / Studio Web Worker) across a frozen multi-prompt panel with fresh-process save/restore continuation.
- Result: cancel probe confirms the parity harness is missing; proceeding.

## Files in scope

- `crates/uor-r4-integer/src/bin/uor-chat.rs` (estimated ~160 lines: add `--prompt-ids`, `--save-session`, and `--restore-session` on the `--stack` serving path using `uor_r4_integer::session_codec::{encode, decode}`)
- `crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs` (estimated ~280 lines: 3-leg library + `uor-chat` CLI + Browser WASM / Studio worker bit-identity and fresh-process `UR4SESS1` snapshot parity test suite across the 8-prompt panel, plus planted-fault controls)
- `web/studio/verify_m7_parity.mjs` (estimated ~140 lines: Node WASM + `uor_model_worker.js` driver for the 8-prompt uninterrupted and `32 + 16` fresh-process restore checks)
- `docs/labs/m7-parity-library-cli-browser/README.md` (estimated ~95 lines: lab record, verification receipts, and staged six-column research notebook row for `m4-serving.md`)
- `docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md` (estimated ~80 lines: frozen input hashes, 8-prompt panel specification, and proposed M7 parity bar for PM registration)
- `docs/products/web-client.md` (estimated ~10 lines: reference the 3-way library/CLI/browser parity harness)

## Done when

- Design note and bar-registration request are posted on Issue `#2867`.
- `uor-chat --stack` supports `--prompt-ids <csv>`, `--save-session <path>`, and `--restore-session <path>` using `UR4SESS1` (`session_codec::encode` / `decode`), with zero softmax (D23), zero matmul (D25), and zero floating-point sampling.
- `cargo test -p uor-r4-integer --features serving-wasm --test m7_parity_library_cli_browser` and `node web/studio/verify_m7_parity.mjs` pass on the `seed = 5` `mk-synthetic-stack` (`UORLUT01`, `469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`), proving bit-identical 48-token trajectories and byte-identical `UR4SESS1` session snapshots (both uninterrupted and `32 + 16` fresh-process save/restore) across the Rust library, `uor-chat` CLI, and Browser WASM / Studio worker, and failing under a planted fault.
- `docs/labs/m7-parity-library-cli-browser/README.md` and `docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md` record the frozen panel hashes, receipts, and staged `m4-serving.md` notebook row (`References #2867`; item remains `status: ready` until an accepted M4 `UORLUT01` artifact exists).

## Stop and ask if

- Any file outside `crates/uor-r4-integer/**`, `web/**`, `docs/products/**`, or `docs/labs/m7-parity-library-cli-browser/**` needs modification.
- Registering the final M7 acceptance bar requires an owner/PM decision (we stage the `DRAFT` pre-registration sheet and post the decision request on `#2867`).
