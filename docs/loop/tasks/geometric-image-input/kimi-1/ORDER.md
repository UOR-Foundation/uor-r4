# ORDER: deterministic integer R4 image decomposition and Studio input

Lab: kimi. Item: [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608), with follow-up [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619). Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## Goal

Accept pictures in the live Studio only through deterministic integer RGBA decomposition into bounded, hash-bound R4 patches. Add the native decomposition route and validated `uor_r4_image` extension, then Canvas paste/drop/picker input with previews and explicit Send. Do not claim visual understanding, useful conversation, a trained multimodal artifact, or production readiness.

## Design

[DESIGN.md](DESIGN.md). One internal vision crate owns decomposition and canonical serialization. The native OpenAI service owns admission, context preflight, and generation. One browser module owns Canvas; existing Studio state remains authoritative.

## Files in scope

- `Cargo.toml`, `Cargo.lock`, `src-tauri/Cargo.lock`, `xtask/src/{main.rs,roadmap_sync.rs}`, new `crates/uor-r4-vision/**`, and narrow `crates/uor-r4-integer/{Cargo.toml,src/openai.rs,src/loopback.rs,tests/openai_loopback.rs}` changes.
- `web/studio/index.html`, `index.css`, `README.md`, `assets/js/**`, and focused `src/adapters/**` tests, including new `image_input.js` and its tests.
- This task folder, `docs/roadmap/items/geometric-image-input.md`, `docs/start/FILEMAP.md`, generated roadmap data when produced by a safe sync, and narrow `docs/products/web-client.md` updates.

## Done when

- Hand-computed 1×1, uniform, asymmetric, odd-dimension, horizontal-edge, vertical-edge, and alpha fixtures prove exact patch rectangles, means, luma/chroma/signs, directional edges, Q12 features, canonical bytes, hash binding, deterministic repeats, controlled pixel changes, and bounded refusals. No float, matmul, semantic caption, or external vision model exists.
- `POST /v1/images/decompose` enforces authenticated loopback, octet-stream, explicit dimensions/grid, exact body length, and 256×256/16-grid bounds. Capabilities distinguish decomposition from visual understanding. Ordinary `image_url` and unsupported endpoint families keep explicit 501 responses.
- Native `uor_r4_image` accepts only schema `uor-r4.image/1` with matching hash and strict canonical prompt, then uses existing context preflight and exact usage/generation. Overflow returns 413 without truncation.
- Browser Canvas accepts paste/drop/picker images, downsamples proportionally to at most 256×256, releases object URLs, and exposes previews/names/sizes/removal plus `R4 patch input — experimental; not visual understanding`. Native decomposition/send occurs only after successful processing and explicit Send. Custom providers may use ordinary `image_url`; local WASM and unsupported cases fail visibly. Image blobs never persist; reload asks for reattachment.
- Vision/native/Studio suites, previous API/composer checks, real decode→decompose→send browser flows with changed-pixel hashes and exact usage, native/WASM audits, repository gates, fresh exact-head review, green GitHub checks, protected queue, and fresh-main verification pass.
- The PR closes #3619 only after full acceptance. #3500 and #3569 remain open. No production publish or M4/M7/useful-chat/visual-understanding/multimodal claim.

## Stop and report conditions

Stop before adding an external runtime dependency, changing model/training/tokenizer/artifact semantics, mounting the typed library, weakening loopback admission or request bounds, silently truncating images/canonical context, persisting image blobs or launch tokens, enabling images without the integer decomposition, adding a captioner/semantic label, publishing production, or editing `.github/**`, another lab's files/issues, or the owner checkout. Merge forward from fresh `main`; never force-push. Reuse the existing Rust/npm runtimes and preserve all unique evidence.
