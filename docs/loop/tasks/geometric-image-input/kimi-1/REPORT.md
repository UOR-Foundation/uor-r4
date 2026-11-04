# REPORT: deterministic integer R4 image decomposition and Studio input

Lab: kimi. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619), dependent on delivered [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Original base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`. Implementation merge head for the measurements below: `4806f2675c50b799c2c0914f822d7a3e7ad98c81`. Final exact head and external delivery checks are recorded on the PR.

## What did not work

- Merging current `main` replaced the earlier JSON/prompt contract with canonical `R4IMG001` bytes but left the browser expecting retired fields and the native parser without server issuance. Failing-first fixtures produced three Studio failures and a missing `issue_decomposition` native API. Two test-only configuration-shadowing errors were corrected before accepting the native RED result.
- The first real current-head pixel-A send returned a truthful 413: the merged verbose R4 text required 206 positions of the artifact's 192-position context. A compact-renderer regression failed on the missing `R4IMG1/1x1/1x1/1:` payload before lossless base64 encoding was added.
- The broad OpenAI suite reproduced a timing-dependent busy-test failure when a 40-token stream finished before the second request. The fixture now holds a generation permit explicitly, asserts 429, releases it, and asserts a normal stream. A disconnect can also report the OS's expected `NotConnected`; that is accepted only for the client-side shutdown, while permit release must still pass.
- Public-head merging exposed fixture and tooling faults: per-server IDs were required, a direct `sendViaApi` fixture lacked its normal user turn, the API stub did not await content promises, `readTextFiles` was missing from the page imports, the merged verifier had only Linux browser paths and raced boot, and synthetic file events did not exercise Chrome's real file-input path. These were localized with CDP evidence and repaired without weakening the application assertions.
- The complete integer run passes its library and binaries, but six unrelated adversarial token-boundary tests fail because external tokenizer/bundle fixtures are absent. Workspace-wide rustfmt also reports many untouched baseline diffs. Neither unrelated set was rewritten or presented as a pass.
- DeepSeek's #3628 merged while this repair was underway. This branch is a follow-up to that public implementation, not a competing claim that Canvas or deterministic decomposition was first built here.
- A real delayed-browser Stop remains NOT_RUN; the automated native-preparation abort regression passes with outcome `stopped`. No semantic image-quality evaluation was run.

## What changed

- DeepSeek's `uor-r4-vision` and native ingestion remain the upstream base: checked integer RGBA patches, two signed Q12 R4 vectors, canonical `R4IMG001` bytes, Blake3 identity, bounded query/octet-stream transport, and strict canonical parsing. No float, matmul, learned captioner, external vision model, or semantic label exists.
- The native server retains at most 64 issued decompositions in a mutex-protected FIFO registry. `/v1/images/decompose` returns `X-UOR-Decomposition-Id` and exposes it through CORS. A flat native `uor_r4_image` part requires exactly eight fields: `type`, `schema`, `canonical_hex`, `source_blake3`, `width`, `height`, `grid`, and `decomposition_id`. The complete content must match the issued record. Unknown, expired, substituted, and non-user parts fail with 400; valid replacement bytes and recomputed hashes do not bypass issuance.
- Accepted images render as compact base64 containing every big-endian signed Q12 value, prefixed by dimensions/grid/patch count. This is lossless and avoids the 206-position verbose-text overflow without truncating features. The source hash remains validated transport identity rather than prompt text.
- The merged async Studio preparation preserves DeepSeek's public structure. `studio_api.js` validates every identity header, including the issued ID and exact dimensions/grid, checks canonical magic and bounds, and emits the flat part. Per-image abort checks remain. `apiMessages` refuses when only the memory-only reload marker remains, so an old image cannot silently disappear from a send.
- Existing Canvas paste/drop/picker behavior, previews, removal, the exact `R4 patch input — experimental; not visual understanding` label, custom `image_url`, local-WASM refusal, and reattachment behavior are preserved. The merged verifier now supports macOS Chrome, waits for actual page/UI readiness, and uses CDP file input.
- `src-tauri/Cargo.lock` records the internal dependency. The typed library remains unmounted. Scoped roadmap sync filters issue operations and milestone checklist/label edits to the selected item. #3500 and #3569 remain open.

## Measurements and checks

- Studio: **33 files / 278 Vitest tests PASS**, **41 Node tests PASS**, typecheck and Vite build PASS. Studio worker/static/runtime-page and desktop checks PASS with zero unexpected external requests.
- Native: **14/14 OpenAI integration tests PASS**, including valid issuance, unknown/expired IDs, valid-content substitution, non-user roles, image count, context overflow, CORS, and deterministic busy/disconnect behavior. Vision: **9/9 PASS**. xtask: **222/222 PASS**.
- Native library and binaries: **452 library tests PASS, 1 ignored**, plus **17 binary tests PASS**. The six external-fixture adversarial failures are disclosed above and excluded from this count.
- Native serving audit at `4806f2675`: **FULL PASS**, 44 symbol ranges and 130 reachable call-graph functions, zero forbidden instructions. `uor-r4-stack` SHA-256 `19c26c176b677dc35bccdaefe57855f899eb1a10b2aae3b6400fdceb184ba945`. Integer multiply/divide remains within D80/D81.
- WASM serving audit at `4806f2675`: **7/7 PASS**, zero floats or unclassified instructions. Bundle 683,113 bytes, SHA-256 `b57fc32d3c41f0cdb54e0e0a2535d6d85c6216971e5f824ef9693d52aeda07b8`. The same bundle passes Studio worker checks.
- Scoped rustfmt and `git diff --check` PASS. Final repository gates, local policy, exact-head review, GitHub checks, protected queue, and fresh-main verification remain external delivery steps.

## Real browser and native evidence

Used existing dual-c131 artifact SHA-256 `9e80cc6a894ed50c7dc7d0393e2d3b22f74a7230b52d8c635736202a2afa8694`, schema `uor-r4.lut-stack/5`, context 192, with the established W2 tokenizer. Final native `uor-chat` SHA-256 `cad07448ce33544de2bddfa7c15de09e17a5a66b8efa17a5c174ed42a1b6c66c`.

- The merged native browser verifier passes image-only send, exact **31 prompt / 6 completion / 37 total usage**, memory-only reload, and a valid second send with historical image markers.
- Final dual-c131 pixel A returned `I.`: **53 prompt / 3 completion / 56 total tokens**, 172.3 tok/s, TTFT 16.789 ms, elapsed 17.408 ms.
- In a separate clean session, one-byte-changed pixel B returned `I.`: **54 prompt / 3 completion / 57 total tokens**, 169.3 tok/s, TTFT 17.198 ms, elapsed 17.725 ms. The changed pixel changes the R4 payload; no visual-quality inference is made from these tiny fixtures.
- The UI displays previews and the exact experimental label. Reload reports that image attachments were not restored and shows zero attached files. Send without reattachment then refuses visibly: `Reattach the previous image after reload; image bytes are not persisted.` Browser tab and native server were stopped.
- The earlier 206/192 failure is retained as a negative result, not replaced in the record. A real delayed-browser Stop remains NOT_RUN.

## Roadmap and delivery notes

- `main` contains DeepSeek's #3628 Studio implementation and #3630 evidence repair. This follow-up adds issuance, schema alignment, substitution/expiry coverage, lifecycle fixes, and current-head verification. The PR references #3619 and #3628 rather than duplicating their closure; #3500/#3569 also remain open.
- Final roadmap/issue state must be read from fresh `main` before any scoped sync. No unrelated issue edits are allowed.
- No production publish, paid provider, useful-chat, production-readiness, M4/M7 completion, trained multimodal, captioning, recognition, or visual-question-answering claim is made.
- Modern Web Guidance `accessibility` informed decorative preview `alt=""` and dynamic error/live-region semantics. No Canvas-specific guide matched; existing native Canvas/fetch patterns were retained.

## Checks run at the head

The implementation measurements used `4806f2675c50b799c2c0914f822d7a3e7ad98c81`; the candidate then merged `35bfa270800e433f4225dbe418d1ecd4be6b28ba` without conflicts. Full repository gates are **11/11 GREEN**. Final local policy and clean-head checks are rerun after every delivery-record edit; GitHub checks, exact-head review, protected queue, and fresh-main verification are external records on the PR.

## Skills used

Isolated worktrees, plan execution, test-driven development, systematic debugging, solution integrity, modern web accessibility guidance, integer Rust decomposition, native loopback/OpenAI protocols, plain-JS Canvas composition, and real built-in-browser verification. No GPU, paid compute, production publication, visual-quality evaluation, or model-quality evaluation was used.

## Still open

- Final repository delivery checks and fresh-main verification remain. Browser Stop was not retested. External-fixture adversarial failures and unrelated baseline formatting findings remain outside this unit.
- #3500 and #3569 remain open. A future trained multimodal artifact and registered evaluation are required for visual question answering.

## Plain summary

Native `uor-chat` and Studio now agree on bounded binary R4 image input. The server issues each decomposition and rejects altered or expired references, both real pixel turns work with exact usage, and reload requires reattachment. This is deterministic representation and transport, not visual understanding.
