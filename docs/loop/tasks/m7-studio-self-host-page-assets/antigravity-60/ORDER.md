# ORDER: m7-studio-self-host-page-assets, antigravity-60

Lab: antigravity. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md). Base: `e6f261600`.

## Goal

The item's done-when reads:

> `web/studio/index.html` and (if kept) the router demo page make zero external requests on load, checked by the extended trap verification; the stack-bound tokenizer artifact replaces the stand-in; the GitHub API calls' position is recorded in the item's record; `docs/products/web-client.md` updated to match.

## Cancel probe (run first)

Ran the page-load and worker verification suite on the clean checkout at `e6f261600e8f6f2cf90afc5659cef9dc8dedeffb`:
1. `node web/studio/verify_studio_page.mjs --strict-router` -> `verdict: "PAGE-LOAD CHECKS PASSED"` (`clean: ["geometric_prime_router_webapp.html", "index.html"]`, `failures: []`).
2. `node web/studio/verify_studio_page_load.mjs --strict-router` -> `verdict: "RUNTIME PAGE-LOAD TRAP PASSED"` (`index.html`: 6 requests, 0 external; `geometric_prime_router_webapp.html`: 2 requests, 0 external; `trapSelfTest`: `plantedExternalDetected: true`, `channels: ["inline", "blob-worker", "late"]`).
3. `node web/studio/verify_studio_worker.mjs` -> **FAILED** (`Error: ENOENT: no such file or directory, open '.../web/studio/pkg/uor_r4_integer_bg.wasm'`), and `web/studio/assets/js/uor_tokenizer_artifact.js` + `web/studio/tools/make_tokenizer.mjs` only implement `mode: "byte-mod"` (`rawBytes[i] % vocabSize`, retaining the `"byte-mod stand-in"` label and mapping rather than replacing the byte-mod stand-in with a bundled stack-bound `ByteLevel` BPE tokenizer artifact and refusing `uor-r4.lut-discrete/1` or reload re-binding).

- **Cancel-probe result:** **DID NOT CANCEL** (`instrument`). The page-load static and runtime traps pass, while `verify_studio_worker.mjs` fails on a clean checkout (`ENOENT` on `pkg/uor_r4_integer_bg.wasm` and `pkg/synthetic-stack.lut`) and the tokenizer leg (`uor_tokenizer_artifact.js`, `make_tokenizer.mjs`, `uor_model_worker.js`) still uses the `byte-mod` stand-in instead of a bundled stack-bound `ByteLevel` BPE tokenizer artifact.

## Plan (expected files > 3)

1. `web/studio/assets/js/uor_tokenizer_artifact.js`: implement the stack-bound `ByteLevel` BPE tokenizer mode (`mode: "byte-bpe"`, GPT-2 `bytesToUnicode`/`unicodeToBytes`, leftmost-longest atomic `added_tokens`, optional `Digits` + `ByteLevel` pre-tokenization, rank-ordered `merges`, piece-to-id encoding, and id-to-bytes decoding) alongside `mode: "byte-mod"`, bundle `DEFAULT_STACK_TOKENIZER_ARTIFACT` bound to the canonical seed-5 `mk-synthetic-stack` digest (`d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `vocab_size: 40`, matching `crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs`), and replace the `"byte-mod stand-in"` label with `"stack-bound byte-bpe tokenizer (vocab N, stack sha256 ...)"` when bound.
2. `web/studio/tools/make_tokenizer.mjs`: support both `uor-r4.lut-stack/1` (`shape.vocab`) and `uor-r4.lut-discrete/1` (`shape.vocabulary`) headers, default to emitting a `mode: "byte-bpe"` stack-bound tokenizer artifact (accepting `--source-tokenizer <hf-tokenizer.json>` for custom/M4 BPE vocabularies and `--mode byte-mod` when explicitly requested), and validate `--check` against both `byte-bpe` and `byte-mod` artifacts.
3. `web/studio/assets/js/uor_model_worker.js`: resolve the active tokenizer artifact per loaded stack (`payload.tokenizerArtifact`, `../../pkg/tokenizer.json`, or `DEFAULT_STACK_TOKENIZER_ARTIFACT`) without stale per-worker caching across reloads, encode text prompts through `encodeTextWithArtifact` (bound `byte-bpe` when matched, unbound `byte-mod` fallback otherwise), and decode generated tokens through `decodeTokenIdWithArtifact`.
4. `web/studio/verify_studio_worker.mjs`: add `ensureWasmAndSyntheticStack()` so `node web/studio/verify_studio_worker.mjs` builds `pkg/uor_r4_integer_bg.wasm` and `pkg/synthetic-stack.lut` automatically on a clean checkout, and verify bound `byte-bpe` prompt encoding, token decoding, reload re-binding, `uor-r4.lut-discrete/1` header support, and unbound fallback.
5. `docs/products/web-client.md`, `docs/labs/m7-studio-self-host-page-assets/README.md`, and `docs/roadmap/items/m7-studio-self-host-page-assets.md`: update the product documentation, lab record, and item status (`status: done` once all four legs pass on a clean checkout; or `status: held` if held on M4 export).

## Files in scope

- `web/**`
- `docs/products/**`
- `docs/labs/m7-studio-self-host-page-assets/**`

## Done when

`node web/studio/verify_studio_page.mjs --strict-router`, `node web/studio/verify_studio_page_load.mjs --strict-router`, and `node web/studio/verify_studio_worker.mjs` all pass on a clean checkout with zero external requests (`remoteRequests === 0`); the stack-bound `ByteLevel` BPE tokenizer artifact (`uor-r4.studio-tokenizer/1`, `mode: "byte-bpe"`) is bundled and loaded in `uor_model_worker.js`, replacing the `rawBytes[i] % vocabSize` stand-in on bound stacks; the GitHub API calls' position is recorded in `docs/labs/m7-studio-self-host-page-assets/README.md` and `docs/products/web-client.md`; and `uor-harness loop gates --full` passes.

## Stop and ask if

- Never wait on Casey or another lab (`CHARTER.md` §5.1, D65–D69). If a status or scope decision is needed, run a fresh-context D68 decision review and log it on `#3075`.
- Stop only if a change requires editing an owner-only path (`CHARTER.md`, `AGENTS.md`, `docs/integration/DECISIONS.md`, `.github/**`).
