# REPORT: repair Studio loading and share exact arithmetic

Lab: kimi. Item: [m7-lexlean-studio-tools](../../../../roadmap/items/m7-lexlean-studio-tools.md) (#3704). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Fresh `origin/main` base: `7018b09cd`. Delivery is **pending**: this report is updated only after exact-head review, protected merge, fresh-main verification and Pages publication.

## What did not work

- **LexLean is not an acceptable default or proof of useful general conversation.** The live full-pack model computed `17 + 25 = 42`, but answered self-description with an unrelated Hunger Games review and answered `What number did I just ask you to compute?` with unrelated Fibonacci code. Its deterministic tools/templates, older `/8` memory layout (`reply_key_len: 6` versus the current `20`), sigmoid tables, heuristic corpus and absent OpenAI endpoint prevent a compatible, charter-lawful drop-in replacement. No external WASM, second runtime, new model format, corpus or default-model change was introduced.
- **The ordinary model remains a synthetic protocol fixture.** Browser output includes synthetic/special-token text and native output is not useful chat. No useful-conversation, visual-understanding, production-readiness, M3/M7/M8, exact sidecar hidden-state resume, signed-distribution, forced-kill cleanup or GitHub-authentication claim is made.
- A worker filename or expected-hash mismatch previously remained pending in `finalizing`; it now becomes an explicit error. The no-storage default remains the bundled synthetic demo and catalog `primary` remains display-only.
- The first native history repair removed only the assistant's long tool trace. A real host + sidecar follow-up still failed with **HTTP 413: 81 positions against 64**. The saved older app session had previously shown 212 positions. The regression was changed to require removal of the whole tool exchange and failed first: the arithmetic user prompt remained beside `Hi`. The paired filter then passed, and the real native follow-up succeeded. Both turns stay visible locally; only subsequent model context excludes them.
- An over-broad root `cargo fmt --all -- --check` reported unrelated historical formatting diffs under `docs/labs/**`; scoped integer and Tauri formatting checks are the product signal. One full default MCP run failed `remote_auth_timeout_and_size_caps_fail_explicitly` with a transport error before its 401 assertion; the focused test, complete MCP suite and subsequent complete default/GUI matrices passed. This is recorded as a transient fixture failure, not hidden or called a mechanism negative.

## What changed

- `m6_exact_arithmetic.rs`: a strict, 160-byte-bounded prompt parser for one unsigned decimal `+` or `-`, with 10-digit operand bounds and explicit malformed, overflow and subtraction-underflow errors. It calls `GeometricArithmeticEngine` and returns answer-first text with actual place-value carry/borrow records; ordinary prose and lookalike `plus` wording fall through.
- WASM and browser worker: `exactArithmetic` returns structured JSON. A handled answer has no model token IDs, no model weights read, zero completion tokens and no invented token rate. The original 32-token trajectory, save/restore and network traps remain intact.
- Tauri and OpenAI transport: `native_exact_arithmetic` shares the Rust function. `desktop_openai.js` returns a JSON fallback with `model: uor-chat`, `uor.tool: m6-exact-arithmetic`, empty completion IDs and zero usage; ordinary prompts retain the existing sidecar stream.
- Conversation history: the exact-tool user prompt and assistant answer remain in the visible session but are removed as one exchange from later OpenAI context. This is the repair for the measured native 81-position failure, not a context-limit increase or silent truncation.
- Identity/package checks: stale operation IDs are ignored, current filename/hash mismatches fail visibly, and the app verifier compares the staged `index.html` and `studio_api.js` against current source to reject stale frontend packaging.
- Product docs, notebook and this report preserve the measured LexLean rejection, actual tool scope and remaining model limitations.

## Measurements

- Studio: **316/316 Vitest**, TypeScript, **69/69 Node**, and real WASM worker checks pass, including exact `42`, zero tool token IDs, original 32-token generation, save/restore and zero remote requests.
- Integer: scoped formatting and the default library pass **80/80**. The serving-feature integration command exits 0 with the explicit missing-fixture exclusions listed below.
- Default Tauri matrix: **7** library, **13** controls, **7** MCP, **13** review and **8** sidecar tests. GUI matrix: **9**, **17**, **8**, **13** and **9** respectively. Both complete commands exit 0.
- Real clean browser session: arithmetic returns `42` with actual carry trace and **0 model tokens**; the following `Hi` completes **32 tokens** at a measured **3,106.8 tok/s**, with no 413. This is one synthetic protocol run, not a quality benchmark.
- Real native host RPC + packaged sidecar: arithmetic returns `42` without model generation; the following `Hi` completes **16 tokens** with exact prompt/completion/total usage **19/16/35**, finish `length`, and no 413. The response text is synthetic and not useful conversation.
- No GPU, paid compute, external API, remote tunnel or new dependency was used.

## Checks run at the head

The serving integration command is:

```sh
CARGO_TARGET_DIR=target/kimi-lexlean/integer cargo test --locked -p uor-r4-integer --features serving-wasm --tests -- \
  --skip test_challenger_ --skip test_m2_challenger_ --skip test_m3_challenger_ \
  --skip test_m2_latency_scaling --skip test_m2_detailed_step --skip test_m2_micro_breakdown \
  --skip test_m1_real_bundle --skip test_m1_prompt_ingestion_clean_boundaries \
  --skip test_m1_role_tokens_fallback_mapping_real_tokenizer --skip test_measure_ \
  --skip test_low_bit_dot_4x --skip test_m2_peak_rss \
  --skip test_quaternion_detailed_token_step_breakdown --skip test_timing_breakdown_step_conversational
```

These are pre-existing tests requiring unavailable local model/tokenizer bundles; they were not renamed, edited or presented as passes. One pre-existing `stack_chat_reference` test remains ignored by its own declaration. The default-feature integration command does not compile the retained serving root and is not the applicable suite; the explicit `serving-wasm` feature is required.

Final package, harness, review, merge, fresh-main and hosted checks are recorded below only after execution.

## Skills used

Project research/delivery workflow, isolated worktrees, test-driven development, systematic debugging, real built-in-browser verification, native host/sidecar calls, package verification and independent exact-head review. No GPU, paid compute, new dependency, model retraining or publisher-code change was used.

## Delivery and cleanup

PENDING. The candidate build started with about **16–17 GiB** free, below the 30 GiB target. Reproducible outputs are confined to `target/kimi-lexlean/**`, `target/hygiene`, `web/studio/{node_modules,pkg}`, `src-tauri/{runtime,binaries,icons,gen}` and temporary publish worktrees. Research notes and unique evidence are preserved. No Trash emptying, model-store deletion, broad worktree cleanup or deletion of another lab's files is authorized.

## Still open

The static Studio and deterministic tool do not make the underlying synthetic model coherent. LexLean weight/model-card licensing remains unresolved, and no LexLean redistribution occurs. Any future model-only export/retrain needs its own item and matched evidence. Protected delivery, final fresh packaged-GUI verification, hosted publication and the final free-space measurement remain pending at this draft.

## Plain summary

LexLean is not the default: its calculator is useful, but its general replies and runtime contract do not qualify. UOR Studio and Tauri now share a bounded exact-arithmetic tool with truthful zero model usage, explicit worker identity errors, and a tested repair that keeps the whole tool exchange out of later model context. Real browser and native arithmetic → ordinary-turn sequences complete without the former 413; the ordinary output remains synthetic, not useful conversation.
