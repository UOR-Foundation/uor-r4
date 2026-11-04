# REPORT: deterministic integer R4 image decomposition and Studio input

Lab: kimi. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619), dependent on delivered [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Original base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`. Final exact head and external verification are recorded on the PR.

## What did not work

- Merging current `main` replaced the earlier JSON/prompt contract with canonical `R4IMG001` bytes but left the browser expecting the retired fields and the native parser without server issuance. Failing-first fixtures produced three Studio failures and a missing `issue_decomposition` native API. Two test-only configuration-shadowing errors were corrected before accepting the native RED result.
- The first real current-head pixel-A send returned a truthful 413: the merged verbose R4 text required 206 positions of the artifact's 192-position context. A compact-renderer regression failed on the missing `R4IMG1/1x1/1x1/1:` payload before the lossless base64 encoding was added.
- The broad OpenAI suite again reproduced a timing-dependent busy-test failure: a 40-token stream finished before the second request, turning expected 429 into 200. The fixture now holds a generation permit explicitly, asserts 429, releases it, and asserts a normal stream; production busy behavior did not change.
- The complete integer run passes 452 library tests and all binary tests with one ignored, but six unrelated adversarial token-boundary tests fail because external tokenizer/bundle fixtures are absent. Those failures are not presented as passes or as image regressions.
- Workspace-wide rustfmt reports many untouched baseline diffs. Only branch-local Rust formatting was corrected; unrelated files were not reformatted.
- DeepSeek's draft #3628 initially duplicated the Studio unit, then merged with its compact renderer while this repair was underway. This branch is now a follow-up to that public implementation, not a competing claim that Canvas or deterministic decomposition was first built here.

## What changed

- DeepSeek's delivered `uor-r4-vision` and native ingestion remain the upstream base: checked integer RGBA patches, two signed Q12 R4 vectors, canonical `R4IMG001` bytes, Blake3 identity, bounded query/octet-stream transport, and strict canonical parsing. No float, matmul, learned captioner, external vision model, or semantic label exists.
- The native server now retains at most 64 issued decompositions in a mutex-protected FIFO registry. `/v1/images/decompose` returns `X-UOR-Decomposition-Id` and exposes it through CORS. A flat native `uor_r4_image` part requires exactly eight fields: `type`, `schema`, `canonical_hex`, `source_blake3`, `width`, `height`, `grid`, and `decomposition_id`. The complete content must match the issued record. Unknown, expired, substituted, and non-user parts fail with 400; valid replacement bytes and recomputed hashes do not bypass issuance.
- Accepted images render as a compact base64 payload containing every big-endian signed Q12 value, prefixed by dimensions/grid/patch count. This is lossless and avoids the 206-position verbose-text overflow without truncating features. The source hash remains validated transport identity rather than prompt text.
- Browser `image_input.js` posts exact RGBA in the query/octet-stream contract with the generation abort signal and `cache: 'no-store'`. It reads canonical binary bytes, validates all identity headers and bounds, emits lowercase canonical hex, and preserves server error messages. `apiMessages` emits the exact flat part. Existing previews, removal, exact experimental label, abort/stopped lifecycle, custom `image_url`, local-WASM refusal, and reload reattachment remain intact.
- `src-tauri/Cargo.lock` records the internal dependency. The typed library remains unmounted. Scoped roadmap sync filters issue operations and milestone checklist/label edits to the selected item. #3500 and #3569 remain open.

## Measurements and checks

- Post-alignment Studio: **33 files / 278 Vitest tests PASS**, **33 Node tests PASS**, typecheck and Vite build PASS. Studio worker/static/runtime-page and desktop checks PASS with zero unexpected external requests.
- Native: **14/14 OpenAI integration tests PASS** after the compact-renderer and deterministic busy-fixture repairs. Vision: **9/9 PASS**. xtask: **222/222 PASS**, including scoped roadmap sync.
- Full integer library/binary run: **452 library tests PASS, 1 ignored**, and every binary test passes. The separate six-fixture-dependent adversarial failures are disclosed above.
- Scoped rustfmt on the branch's Rust files and `git diff --check` PASS. Final repository gates, native/WASM audits, GitHub checks, exact-head review, and fresh-main verification remain external delivery steps and are not claimed here.

## Real browser and native evidence

Used existing dual-c131 artifact SHA-256 `9e80cc6a894ed50c7dc7d0393e2d3b22f74a7230b52d8c635736202a2afa8694`, schema `uor-r4.lut-stack/5`, context 192, with the established W2 tokenizer and a rebuilt native release.

- Current text-only turn returned `I.`: **15 prompt / 3 completion / 18 total tokens**, 387.3 tok/s, TTFT 7.210 ms, elapsed 7.746 ms. ArrowUp restored `Hello native` in the composer.
- The first current-head pixel-A turn reached native preflight and returned the explicit **206 of 192** context error described above. It did not generate or claim visual understanding.
- The compact renderer's real pixel-A/B sends and reload check are **NOT_RUN yet** at this report checkpoint. Earlier image results from other heads are historical and are not reused as current-head evidence.
- A real delayed-browser Stop remains NOT_RUN; the automated native-preparation abort regression passes with outcome `stopped`.

## Roadmap and delivery notes

- `main` now contains DeepSeek's #3628 Studio implementation and #3630 evidence repair. This follow-up preserves those changes and adds issuance, schema alignment, substitution/expiry coverage, and current-head verification. It does not duplicate their authorship or close an already closed item as if it were still open.
- The final PR should reference #3619 and #3628, plus #3500/#3569 without closing them. Final roadmap/issue state must be read from fresh `main` before any sync; no unrelated issue edits are allowed.
- No production publish, paid provider, useful-chat, production-readiness, M4/M7 completion, trained multimodal, captioning, recognition, or visual-question-answering claim is made.

## Skills used

Isolated worktrees, plan execution, test-driven development, systematic debugging, solution integrity, modern web accessibility guidance, integer Rust decomposition, native loopback/OpenAI protocols, plain-JS Canvas composition, and real built-in-browser verification. No GPU, paid compute, production publication, visual-quality evaluation, or model-quality evaluation was used.

## Still open

- Compact-renderer real image turns, final audits/gates, exact-head review, protected delivery, and fresh-main verification remain to be completed. Browser Stop was not retested.
- #3500 and #3569 remain open. External-fixture adversarial failures and unrelated baseline formatting findings remain outside this unit.

## Plain summary

The native and browser protocol now agree on canonical binary R4 image input, and the server rejects content that was not actually issued to that client. Tests pass, and native text chat works in the real browser. Compact image turns still need their final real-browser verification before this unit can be called delivered.
