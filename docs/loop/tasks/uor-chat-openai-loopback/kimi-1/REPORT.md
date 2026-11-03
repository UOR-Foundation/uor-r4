# REPORT: Native uor-chat OpenAI-compatible loopback service

Lab: kimi. Item: [uor-chat-openai-loopback](../../../../roadmap/items/uor-chat-openai-loopback.md) (#3595). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Base: `de21884d104391b3dfc29e569ca9c33e85f61057`; current-main integration through `45fad9191` was merged without conflicts. Implementation head: `f371e05d51ebec597c6128a29a741119e9040c6d`; the final delivery head is recorded on the PR after the evidence commit.

## What did not work

- The first streaming-parity test failed to compile because `reply_history_with` did not exist. A larger second-turn fixture then exceeded the existing six-token context; its declared context was increased to 16 without changing the engine. The shared callback now passes batch/text/history parity and disconnect-before-next-step tests.
- The initial loopback run rejected every Host because port `0` was used after binding instead of the selected ephemeral port. The live config now records the bound port in both server modes.
- Non-stream fixtures initially requested stream-only usage; a long prompt used bytes absent from the tiny fixture vocabulary; and a wrong-Host fixture connected after shutdown. Those fixture defects were corrected without loosening production validation.
- A valid assistant history containing synthetic whitespace was rejected by an overbroad encoder-count check. Empty user/system turns and all special-token/unknown-role injections remain rejected; assistant whitespace is accepted.
- A SHA-256 token fallback introduced a debug-only `fmov` path. The fallback was removed and token creation now fails closed if `/dev/urandom` cannot be read. The release native audit, which is the declared serving target, passes. A later debug-only SHA-256 precondition was not mistaken for a serving release defect.
- The M7 harness first invoked a non-Rust `cargo` shim. All WASM/worker/parity checks were rerun with the real stable Rust toolchain first in `PATH` and Node retained in `PATH`.
- `verify_studio_worker.mjs` expected an older API import line. The verifier now accepts the already-shipped `normalizeBaseUrl` import; this is the only Studio change and is explicitly test-only.
- A CRLF-injection regression initially passed origin configuration. The server now accepts only nonempty HTTP(S) origins without control characters; exact valid-origin behavior is also tested.
- The unfiltered integer suite has pre-existing local-fixture failures: adversarial token-boundary tests, challenger M2/M3 tests, `diagnostic_slot_scaling`, and other tests with names containing `challenger` or `test_m2_` require absent local bundles/tokenizers. They are reported separately, not relabeled as mechanism negatives. M7 browser parity initially failed only on the cargo shim and now passes.
- No useful-chat, production, M4/M7, MCP, typed-shell, visual-understanding, embedding, or multimodal capability is claimed. Image parts and other OpenAI families return explicit 501 responses.

## What changed

- `StackChat::reply_history_with` is the single greedy generation path for existing batch replies and new per-token emission. `reply_history` delegates with a no-op; IDs, text, state and stop rules are unchanged. Callback errors stop before another model step.
- `openai.rs` binds one loaded non-discrete artifact, validates route-specific fields and content parts, preflights context, computes exact usage/integer timing, maps JSON/SSE responses and errors, buffers UTF-8 losslessly, and enforces an RAII one-generation permit.
- `loopback.rs` provides bounded std-TCP HTTP, exact Host/Origin/token admission, constant-time token comparison, JSON and real SSE, disconnect abortion, 400/404/413/429/501 failures, sequential generation, and optional traversal/symlink-safe same-origin Studio assets with an ephemeral in-memory bootstrap.
- `uor-chat --stack … --tokenizer … --serve 127.0.0.1:<port> [--studio-root …] [--origin …]` loads the same artifact directly. No CLI subprocess, retired router, external runtime, token printing or persistent credential is used. REPL behavior is preserved and incompatible server options fail explicitly.
- Roadmap item #3595 and the owner's request are registered. The only verifier change is the test-only API-import expectation described above.

## Measurements and checks

- Shared stream tests: **2 passed**; existing stack-chat regression group: **14 passed**.
- `openai_loopback` integration: **8 passed**, covering JSON/SSE identity, multi-turn history, exact usage/timing, EOS/cycle/max mapping, text parts, model list/detail/capabilities, legacy completions, Responses, UTF-8 boundaries, malformed/unsupported inputs, Host/Origin/token/body limits, busy refusal, disconnects, static bootstrap/traversal, and the real CLI launch without token output.
- Integer suite with only the identified pre-existing local-fixture exclusions (`challenger` and `test_m2_`): all executed tests passed, including **448 library tests, 1 ignored** and every integration binary. The unfiltered fixture failures remain disclosed above rather than being presented as a full-suite pass.
- Release native serving audit: **FULL PASS**, 96 symbol ranges and 240 reachable functions, zero forbidden instructions. WASM audit: **7/7 PASS**, zero forbidden floating instructions; integer multiply/divide remains allowed only under D80/D81.
- Studio worker: **ALL STUDIO WORKER CHECKS PASSED**, including save/restore identity, guarded reset and zero remote requests. M7 browser parity: **8/8 rows matched**, with zero remote requests.
- Real non-discrete smoke: existing Claude dual-c131 FBS artifact `local/claude/claude-181/exp/raw-both.uorlut` (read-only substitution; the user's previous invocation was not identified) plus W2 tokenizer. Artifact SHA-256 `9e80cc6a894ed50c7dc7d0393e2d3b22f74a7230b52d8c635736202a2afa8694`, schema `uor-r4.lut-stack/5`, declared context 192. Model discovery returned that exact hash/schema. JSON and SSE both returned `I.` with 3 completion IDs, 12 prompt tokens and 15 total tokens. JSON measured `elapsed_ns=5,154,500`, `first_token_ns=4,720,500`, `tokens_per_second_milli=582,015`; SSE independently measured `5,142,709`, `4,671,834`, and `583,350`. `/v1/embeddings` returned 501 and a 10,000-byte prompt returned 413 before streaming. This is protocol evidence only, not a quality claim. The launch token was not printed or persisted, and the server was stopped.
- Storage: 35,506 generated files (6.5 GiB) were removed from the Kimi target after the smoke run. Free space increased from 27 GiB to 32 GiB. No model evidence or other labs' files were deleted.

## Checks run at the head

The implementation and this REPORT share head `f371e05d51ebec597c6128a29a741119e9040c6d`. Final clean-head `loop gates --full`, local policy, claim wording, roadmap check, fresh exact-head review, GitHub checks and protected queue delivery are external verification records and will be posted on the PR only after they exist. They are not claimed in this REPORT.

## Still open

- #3595 stays open until protected delivery and fresh-main verification complete. #3500 typed migration and #3569 MCP behavior remain open and are not changed by this unit.
- Studio prompt history/paste/drop files/composer metrics are the next unit. Integer R4 image decomposition precedes any image input; this unit returns 501 for images and embeddings. Production publication and multimodal qualification are not included.

## Plain summary

The native `uor-chat` stack now has a real loopback OpenAI-compatible JSON/SSE service with exact usage, measured timing, secure local admission and truthful unsupported-route failures. Batch and stream generation share one implementation, real native/WASM audits pass, and a labeled dual-c131 smoke proves the shipped path. No useful-chat or production claim is made.
