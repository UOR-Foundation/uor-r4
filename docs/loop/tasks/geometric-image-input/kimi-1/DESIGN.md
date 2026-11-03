# DESIGN: deterministic integer R4 image input

## Boundary

This unit adds a deterministic representation and transport, not vision. The existing native `IntegerStackModel`/`StackChat`, model artifact, tokenizer, and training semantics do not change. The new internal `uor-r4-vision` crate has no dependency on the integer engine; the engine depends on it, not the reverse. Browser decoding stays in browser-native Canvas. No external runtime dependency or production publication is added.

## Integer decomposition

`decompose_rgba(width, height, rgba, grid)` accepts dimensions 1–256, grid 1–16 no larger than either dimension, and exactly `width * height * 4` bytes. All products, sums, divisions, and conversions are checked. Alpha is composited over black per channel as `(channel * alpha + 127) / 255`. Pixel luma is `(77*r + 150*g + 29*b + 128) / 256` using integer division.

Patches partition the image by `x0 = column * width / grid`, `x1 = (column + 1) * width / grid` and equivalent row boundaries, so odd dimensions cannot lose pixels or produce empty patches. Mean RGB and mean luma use rounded integer means `(sum + count/2) / count`. `chroma_rg = mean_r - mean_g`; `chroma_by = mean_b - mean_luma`. Horizontal edge is the rounded mean absolute luma difference across horizontal adjacent pairs inside the patch; vertical edge is the equivalent across vertical pairs, with zero when no pair exists. R4 edge strength is `max(horizontal, vertical)` while both directional values remain in the patch record. Q12 features are `[luma, chroma_rg, chroma_by, edge_strength] * 16`, preserving signs and four-dimensional positions without floating point or matrix multiplication.

The source hash is SHA-256 over the domain separator `R4IMG1\0`, width/height/grid as little-endian u32, and the original RGBA bytes. `canonical_prompt()` is exactly:

`R4IMG1;sha256=<64-lowercase-hex>;size=<width>x<height>;grid=<grid>;q12=<row-major-comma-separated-features>`

Row-major patch order preserves row/column positions implicitly; the JSON patch records also carry explicit rectangles. A strict parser accepts only this canonical form, matching source hash, bounded dimensions/grid, and exactly `grid * grid * 4` signed Q12 values. It rejects arbitrary image-caption text and malformed/trailing content.

## Native ingestion

`POST /v1/images/decompose` requires `application/octet-stream` plus `X-UOR-Width`, `X-UOR-Height`, and `X-UOR-Grid`. Existing loopback authentication, Host/Origin checks, and body limits remain. The route enforces exact RGBA length and the same 256/16 bounds, then returns the complete decomposition and canonical prompt. Capabilities distinguish decomposition from visual understanding and list this route; image generation, embeddings, and ordinary native `image_url` remain unimplemented.

The only native image content-part extension is `uor_r4_image` with schema `uor-r4.image/1`, `source_sha256`, `canonical_sha256`, `canonical_prompt`, and `decomposition_id`. The route stores at most 64 issued records in memory and expires them on restart. The adapter requires the exact server-issued ID, source hash, canonical hash, and canonical prompt, so recomputing a checksum after changing Q12 cannot forge issuance. It then uses the existing tokenizer/context preflight and generation path. Image parts are rejected in system/assistant history. Oversize context returns 413; no canonical text is silently shortened.

## Browser behavior

`image_input.js` uses object URLs and Canvas to decode a selected/pasted/dropped image, downsample proportionally to at most 256×256, extract RGBA, and release resources on every path. It refuses undecodable/oversized input loudly. The browser chooses `min(2, width, height)` as the default grid to keep the canonical prompt compact; callers may request another valid grid.

Attachments retain one owner in `window.__studioUi`. Image previews and blobs remain in memory; only metadata/decomposition needed for the active send is retained, and reload displays a reattach request rather than pretending blobs persisted. Native mode posts RGBA to the same-origin decomposition route with the generation abort signal, checks it between images, and sends `uor_r4_image` only after successful decomposition and explicit Send; an abort finishes as stopped rather than an error. Custom OpenAI providers may send ordinary data-URL `image_url` content parts and surface provider errors verbatim. Local WASM refuses image context explicitly.

The visible label is exactly `R4 patch input — experimental; not visual understanding`. Nothing in the UI or docs claims captioning, recognition, question answering, or a trained multimodal artifact.

## Verification

Write failing hand-computed tests for 1×1, uniform, asymmetric, odd-dimension, horizontal-edge, vertical-edge, alpha, canonical bytes/hash, controlled pixel changes, deterministic repeats, and all bounds before implementing the crate. Add native route/content-part failure and parity tests, then browser module tests before UI integration. Run all previous native/Studio checks, native/WASM audits, repository gates, and real built-in-browser decode → decompose → explicit send with exact usage and different-pixel hash changes. Deliver through fresh exact-head review, green GitHub checks, protected queue, and fresh-main verification.
