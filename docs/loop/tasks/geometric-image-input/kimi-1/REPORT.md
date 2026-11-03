# REPORT: deterministic integer R4 image decomposition and Studio input

Lab: kimi. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619), dependent on delivered [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`. Final exact head and external verification are recorded on the PR.

## What did not work

- The vision tests initially had 24 missing-API compile failures. After the first implementation, two independent test oracles were wrong: one mixed x/y rectangle fields and one used RGB rather than luma differences for edges. Those expectations were corrected to the designed coordinates and luma-derived 164/92 values; production was not tuned to them.
- Native image tests initially returned 501. Two negative fixtures then proved invalid rather than the server: four ASCII bytes are valid RGBA, and an all-zero canonical grid was too compressible to overflow the 64-position fixture. The tests now use a wrong-length body and four valid image parts to prove 413.
- The pre-existing HTTP busy test failed twice in parallel because the first 40-token stream finished before the second request; it passed in isolation. The same HTTP 429 assertion now runs while holding the server permit, followed by a normal stream, removing timing dependence without weakening coverage.
- The first real browser image send failed with `at least one message is required`. `normaliseMessages` dropped every content-part array. A failing-first API regression now preserves native/custom parts unchanged, and the real send works.
- The first reattach send stayed at `streaming…` because preparation threw before request IDs were captured and outside the `try`. A failing-first lifecycle regression now drives the same reattach error through correlated cleanup. The real browser shows the explicit error and no hanging generation.
- Whole-crate Clippy with `-D warnings` reports 121 pre-existing findings in untouched integer files; those unrelated sources were not rewritten. The new vision crate is Clippy-clean and the touched native files produced no new findings.
- The first WASM build used the host's third-party `cargo` shim and failed on `cargo metadata`. Rebuilding with rustup Cargo on PATH succeeded; no source workaround was added.
- Browser Stop was not retested in this unit and remains outside its evidence. No visual understanding, useful-chat, production, M4/M7 completion, trained multimodal artifact, captioning, recognition, or visual question answering is claimed.

## What changed

- New internal `uor-r4-vision` crate: checked integer RGBA alpha composition over black, luma, rounded patch means, two chroma axes, directional edges, signed Q12 features, odd-dimension partitions, source SHA-256, and strict versioned canonical prompts. Dimensions are bounded to 256 and grids to 16. No float, matmul, learned captioner, external vision model, or semantic label exists.
- Native `/v1/images/decompose` enforces octet-stream, authenticated loopback, explicit `X-UOR-Width`/`Height`/`Grid`, exact body length, and the same limits. Capabilities distinguish decomposition from visual understanding. `uor_r4_image` accepts only schema `uor-r4.image/1`, matching hash, and exact canonical prompt before ordinary context preflight. Ordinary native `image_url` and other unsupported families remain 501.
- Browser `image_input.js` supports raster paste/drop/picker through Canvas, proportional 256×256 downsampling, RGBA transport, object-URL/bitmap cleanup, and visible decode errors. The composer shows previews, names/sizes, removal, and `R4 patch input — experimental; not visual understanding`.
- Native preparation occurs only on explicit Send inside the guarded async lifecycle. Custom providers use ordinary `image_url`; local WASM refuses images. Image bytes/previews stay in memory, only a reattach marker persists, and reload produces an explicit reattach-required placeholder and error rather than a pretend image.
- `src-tauri/Cargo.lock` and the shared `Attachment` type record only the new internal dependency/data shape. The typed library remains unmounted; #3500 and #3569 remain open.

## Measurements and checks

- Vision: **9/9 unit + doc tests PASS**, formatting and Clippy `-D warnings` PASS. Fixtures cover exact 1×1 bytes/hash, uniform/asymmetric/odd/horizontal/vertical/alpha values, Q12 signs, deterministic repeat, controlled pixel changes, strict canonical validation, and bounds/overflow.
- Native: **14/14 OpenAI integration tests PASS** and **451 library/binary tests PASS, 1 ignored**. Exact decomposition, changed hashes, capabilities, strict extension errors, context overflow, and all previous routes/admission are covered.
- Studio: **33 files / 276 Vitest tests PASS**, **30 Node tests PASS**, typecheck and Vite build PASS. Studio worker/static/runtime page checks and desktop checks PASS with zero unexpected external requests. Custom `image_url` and native R4 content parts survive normalization; provider errors remain verbatim.
- Native serving audit: **FULL PASS**, 96 symbol ranges and 239 reachable functions, zero forbidden instructions; integer multiply/divide remains within D80/D81. Updated release SHA-256 `197bd998525dcd7b0aed49a9a04f0935fae897e6d526ae280263649bdd506b76`.
- Updated WASM: **7/7 serving audit PASS**, zero floats; 675,518 bytes, SHA-256 `3439aa1023e6b9346bc29fdc3d4b92153d40415e2fa5c7009df15b98cd91df09`. Studio worker passes against this exact bundle.

## Real browser and native evidence

Used the existing dual-c131 artifact SHA-256 `9e80cc6a894ed50c7dc7d0393e2d3b22f74a7230b52d8c635736202a2afa8694`, schema `uor-r4.lut-stack/5`, context 192, with the established W2 tokenizer and updated native release.

- Real file-dialog upload of scratch 1×1 `pixel-a.bmp` showed a preview and the exact experimental label. Sending `Hi` returned `I.`: **3 completion / 117 prompt / 120 total tokens**, 63.9 tok/s, TTFT 45.646 ms, elapsed 46.958 ms.
- Removing A and uploading one-byte-changed `pixel-b.bmp` then sending `Hi` returned `I.`: **3 / 134 / 137**, 77.9 tok/s, TTFT 37.843 ms, elapsed 38.514 ms.
- Direct native decomposition returned different source hashes: A `78d46b99b22a762639ee0c3ec19b4b8e886b2a616ae9f72b21972a3338cb6e59`; B `d1827a84bd963254d848651b558b5b98b92340d0544556143167059be1f62dbb`.
- Reload showed `Previous image — reattach required`; Send produced the visible error `Reattach Previous image — reattach required after reload; image bytes are not persisted.` No bytes or launch token were printed or persisted. Browser tab and native server were stopped.
- Scratch BMPs were generated outside git and are not corpus/evidence of visual understanding. No image-quality or semantic score was run.

## Roadmap and delivery notes

- The global roadmap sync dry run proposed one intended create plus unrelated updates to #3506, #3407, #3392 and the M7 checklist. Only #3619 was created, with normal labels/milestone and an adoption marker; no unrelated issue or generated mapping was edited. A maintainer-scoped sync remains necessary.
- Final repository gates, local policy, fresh exact-head review, GitHub checks, protected queue, and fresh-main verification are external records and will be posted only after they exist.
- Modern Web Guidance `accessibility` informed decorative preview alt text and dynamic error semantics. No Canvas-specific guide matched; existing native Canvas/fetch patterns and live error regions were retained.

## Checks run at the head

The original implementation is committed through `4b54042812f0ac9fbefa0f765dbd43ef61874bb6`; final repair-head identity and all delivery checks are external verification records on the PR. The measured unit/native/Studio/browser/audit results above are actual command output; protected GitHub delivery is not claimed here.

## Skills used

Isolated worktrees, plan execution, test-driven development, systematic debugging, solution integrity, modern web accessibility guidance, integer Rust decomposition, native loopback/OpenAI protocols, plain-JS Canvas composition, and real built-in-browser verification. No GPU, paid compute, production publication, visual-quality evaluation, or model-quality evaluation was used.

## Still open

- Browser Stop was not retested here. No production publish or visual-understanding/useful-chat claim is made. A future trained multimodal artifact and registered evaluation are required for image question answering.
- #3500 and #3569 remain open. The generated roadmap issue mapping still needs a maintainer-scoped sync. Unrelated whole-crate Clippy findings remain outside this unit.

## Plain summary

Studio can now accept raster pictures and turn them into deterministic integer R4 patches before sending them to native `uor-chat`. Text and image messages work with exact usage, changed pixels change hashes, and reload requires reattachment. This is representation and transport, not visual understanding.
