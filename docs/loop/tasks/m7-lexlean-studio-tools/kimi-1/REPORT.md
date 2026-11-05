# REPORT: repair Studio loading and share exact arithmetic

Lab: kimi. Item: [m7-lexlean-studio-tools](../../../../roadmap/items/m7-lexlean-studio-tools.md) (#3704). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Fresh `origin/main` base: `3f08ccfd1`, including registration PR #3730. Delivery is **pending**: this report is updated only after exact-head review, protected merge, fresh-main verification and Pages publication.

## What did not work

- **LexLean is not an acceptable default or proof of useful general conversation.** Repository commit `23436bdcd69e5238fcda51ef2529bbd4a5dc1e73` and deployed tree `563a7549e432fc035bc603504bb3a162eb332212` identify the inspected source. The live full-pack model computed `17 + 25 = 42`, but answered self-description with an unrelated Hunger Games review and answered `What number did I just ask you to compute?` with unrelated Fibonacci code. Its deterministic tools/templates, older `/8` memory layout (`reply_key_len: 6` versus the current `20`), sigmoid tables, heuristic corpus and absent OpenAI endpoint prevent a compatible, charter-lawful drop-in replacement. No external WASM, second runtime, new model format, corpus or default-model change was introduced.
- **The ordinary model remains a synthetic protocol fixture.** Browser output includes synthetic/special-token text and native output is not useful chat. A third ordinary turn after restoring a synthetic reply is rejected with **HTTP 400 for literal special-token injection**, already documented in the web-client contract. This is separate from exact-tool history filtering and is not presented as working multi-turn conversation. No useful-conversation, visual-understanding, production-readiness, M3/M7/M8, exact sidecar hidden-state resume, signed-distribution, forced-kill cleanup or GitHub-authentication claim is made.
- A worker filename or expected-hash mismatch previously remained pending in `finalizing`; it now becomes an explicit error. The no-storage default remains the bundled synthetic demo and catalog `primary` remains display-only.
- The first native history repair removed only the assistant's long tool trace and still failed **HTTP 413: 81 positions against 64**. After merging `main`, a fresh GUI sequence exposed a second metadata-loss path and failed with **212 positions against 64**: `apiMessages()` discarded the exact-tool marker before the filter could see it. New regressions against the real `apiMessages()` and `loadConversations()` functions failed first with the marker `undefined`, then passed after both serializers preserved it. Both turns remain visible locally; only subsequent model context excludes them.
- An initial packaged build without `--features gui` produced a CLI-only 2.0 MB executable that exited instead of opening a window. Rebuilding the same source with `--features gui` produced the expected 17.8 MB WebKit/AppKit app and the real two-turn sequence. This was a build-command error, not a code fix.
- Broad formatting checks reported unrelated diffs under `docs/labs/**` and in two incoming Claude integer files. Only the changed Rust files are used for scoped formatting evidence. One earlier full default MCP run failed `remote_auth_timeout_and_size_caps_fail_explicitly` with a transport error before its 401 assertion; the focused test, complete MCP suite and subsequent complete default/GUI matrices passed. These are instrument results, not mechanism negatives.

## What changed

- `m6_exact_arithmetic.rs`: a strict, 160-byte-bounded prompt parser for one unsigned decimal `+` or `-`, with 10-digit operand bounds and explicit malformed, overflow and subtraction-underflow errors. It calls `GeometricArithmeticEngine` and returns answer-first text with actual place-value carry/borrow records. No-space `17+25` is normalized and exact; bare numbers, ISO dates, ordinary prose and lookalike `plus` wording fall through to the model.
- WASM and browser worker: `exactArithmetic` returns structured JSON. A handled answer has no model token IDs, no model weights read, zero completion tokens and no invented token rate. The original 32-token trajectory, save/restore and network traps remain intact.
- Tauri and OpenAI transport: `native_exact_arithmetic` shares the Rust function. `desktop_openai.js` returns a JSON fallback with `model: uor-chat`, `uor.tool: m6-exact-arithmetic`, empty completion IDs and zero usage; ordinary prompts retain the existing sidecar stream.
- Conversation history: `apiMessages()` and `loadConversations()` now preserve the exact-tool marker. The user prompt and assistant answer remain visible but are removed as one exchange from later OpenAI context, including after restore. This addresses the measured 81- and 212-position failures without increasing context limits or silently truncating ordinary history.
- Worker identity: current-operation validation no longer depends on a numeric message ID. No-ID discrete responses, missing filenames and missing expected hashes fail closed with an explicit error instead of remaining pending or becoming ready; stale numeric IDs remain ignored. Valid worker responses keep their original readiness behavior.
- Identity/package checks: stale operation IDs are ignored, current filename/hash mismatches fail visibly, and the app verifier compares staged `index.html`, `studio_api.js`, `desktop_openai.js` and `uor_model_worker.js` against current source to reject stale frontend packaging. Packaged verification uses `cargo tauri build --bundles app --features gui` and rejects the older stale bundle as a negative control.
- Scope correction: a fresh D68 decision review approved exactly `web/studio/index.html`, `web/studio/assets/js/studio_api.js`, `web/studio/assets/js/studio_api_streaming.test.mjs`, and `scripts/studio/verify-tauri-app.sh` as minimal authorization for the measured history/package repair. This is scope authorization only; the item remains `ready` and the PR uses `References #3704`, with no M7, bar, cap, owner-file or broader-path decision.
- Product docs, notebook and this report preserve the exact LexLean repository/deployed-tree identities, measured rejection, actual tool scope, new regression evidence and remaining model limitations. Generated roadmap/STATE omissions remain blocked by the pre-existing `infra-desktop-client-tauri` status/mapping mismatch; live issue #3704 is OPEN with `roadmap:item` and the matching body marker, and no generated file was hand-edited.

## Measurements

- Studio after all review fixes: **318/318 Vitest**, TypeScript, **71/71 Node**, and real WASM worker/static checks pass. The two exact-tool metadata regressions fail before the fix and pass after it; the no-space/bare-number/ISO-date arithmetic cases and no-ID/missing-identity worker cases also fail first and then pass. Exact `42`, zero tool token IDs, original 32-token generation, save/restore and zero remote requests are covered.
- Integer: scoped `rustfmt --check` has a passing receipt; the default library and serving-feature integration command exit 0 with the explicit missing-fixture exclusions listed below.
- Default Tauri matrix: **7** library, **13** controls, **7** MCP, **13** review and **8** sidecar tests. GUI matrix: **9**, **17**, **8**, **13** and **9** respectively. One parallel MCP transport attempt failed before assertions, the focused MCP suite passed, and both complete matrices then passed serially with `--test-threads=1`.
- Fresh browser chat at the reviewed head: no-space `17+25` returns `42` with actual carry trace and **0 model tokens**; the following `Hi` completes **32 tokens** at a measured **2,388.1 tok/s**, with no 413. This is one synthetic protocol run, not a quality benchmark. Native RPC returns the same `42`; ISO date `2026-11-05` returns `handled:false` and falls through.
- Real host RPC and packaged GUI: arithmetic returns `42` without model generation. A fresh native GUI chat then completes `Hi` with **16 tokens**, exact prompt/completion/total usage **19/16/35**, finish `length`, and no 413. Before the metadata fix, the same new-chat sequence failed with **212/64**.
- The extra restored third-turn probe remains rejected with **HTTP 400 special-token injection** because ordinary synthetic output includes `<|bos|>`. This is retained as a negative limitation, not hidden or repaired out of scope.
- Packaged contract, native runtime, stale-executable negative control and full harness gates pass. A planted staged `desktop_openai.js` mismatch is rejected, and the restored source passes. No GPU, paid compute, external API, remote tunnel or new dependency was used.

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

The packaged GUI command is `CARGO_TARGET_DIR="$PWD/target/kimi-lexlean/tauri" cargo tauri build --bundles app --features gui`. Full 11/11 harness gates are rerun at the corrected head. Exact-head review, protected merge, fresh-main and hosted checks remain to be recorded after execution.

## Skills used

Project research/delivery workflow, isolated worktrees, test-driven development, systematic debugging, real built-in-browser verification, native host/sidecar calls, package verification and independent exact-head review. No GPU, paid compute, new dependency, model retraining or publisher-code change was used.

## Delivery and cleanup

PENDING. The candidate build started with about **16–17 GiB** free and current free space is measured again before publication, below the 30 GiB target. Reproducible outputs are confined to `target/kimi-lexlean/**`, `target/hygiene`, `web/studio/{node_modules,pkg}`, `src-tauri/{runtime,binaries,icons,gen}` and temporary publish worktrees. Research notes and unique evidence are preserved. No Trash emptying, model-store deletion, broad worktree cleanup or deletion of another lab's files is authorized.

## Still open

The static Studio and deterministic tool do not make the underlying synthetic model coherent or safe for unrestricted multi-turn replay. LexLean weight/model-card licensing remains unresolved, and no LexLean redistribution occurs. Any future model-only export/retrain needs its own item and matched evidence. Protected delivery, hosted publication and the final free-space measurement remain pending at this draft.

## Plain summary

LexLean is not the default: its calculator is useful, but its general replies and runtime contract do not qualify. UOR Studio and Tauri now share a bounded exact-arithmetic tool with truthful zero model usage, explicit worker identity errors, and a red→green metadata repair that keeps the whole tool exchange out of later model context. Real browser and packaged-GUI arithmetic → ordinary-turn sequences complete without the former 413; ordinary output remains synthetic, and restored special-token replay can still return 400.
