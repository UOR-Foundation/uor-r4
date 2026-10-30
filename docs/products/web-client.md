# The web client

The web client is meant to put a language model in a browser tab: open a page once, download the
model, then run it locally in WebAssembly with no model server. The Studio interface and the integer
worker already exist and are connected **on a synthetic stand-in**. No qualified model is bundled,
and accepted-model M7 parity is not established. **M1 is complete, but the web client is still a
demonstration, not a product.**

Labels: **proof**, **measurement**, **hypothesis**. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md) and the [glossary](../start/glossary.md).

![A browser window with three layers: the Studio interface, a Web Worker holding the integer engine, and browser storage for the model and saved sessions. A crossed-out model-server arrow shows that prompts do not leave the device.](../figures/products/web-client-architecture.svg)

*The intended design: one verified download, then local computation in the tab.*

## How it is meant to work

1. **Load the static Studio.** The charter places it at `/studio` on the documentation site.
2. **Download one `UORLUT01` file and verify its SHA-256.** A release pin identifies the model. A
   download-once browser-storage cache is planned, not claimed as delivered.
3. **Start the integer engine in a Web Worker.** The worker loads `pkg/uor_r4_integer.js` and calls
   `init` and `loadStack` on the model bytes.
4. **Generate.** Each token calls `step`. The D11 engine uses integer arithmetic; integer multiply
   and divide are allowed, while floats, transcendentals and dense matmul stay banned. Accepted-model
   cross-browser parity is an M7 measurement, not an assumption.
5. **Save and resume.** `saveSession` emits a `UR4SESS1` snapshot and `restoreSession` loads it.

The real exports are `init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`,
`weightsPerToken`, `allocEvents` and `leakedModels`
([`wasm_bindgen_exports.rs`](../../crates/uor-r4-integer/src/wasm_bindgen_exports.rs)). Serving an
exported D11 artifact on a client device is allowed; the GPU-only research-compute rule does not
apply to product serving ([charter](../../CHARTER.md), section 3.6).

## What exists today

| Piece | Current state | Source |
|---|---|---|
| Studio interface | Editor, file tree, chat UI, model manager and local folder access, with self-hosted vendor shims | [`web/studio/index.html`](../../web/studio/index.html) |
| Integer worker connection | `loadStack`, `step`, save/restore, header/engine-derived capabilities and a stack-bound ByteLevel BPE tokenizer, verified on a synthetic stand-in | [#2868 record](../labs/m7-studio-connect-wasm-engine/README.md) |
| Offline assets and tokenizer | Main page and router demo make zero external page-load requests; the tokenizer artifact replaces the old byte-mod fallback on bound stacks | [#3075 record](../labs/m7-studio-self-host-page-assets/README.md) |
| WebAssembly integer engine | `serving-wasm` build and 7-of-7 serving-function audit; save/restore identity on the synthetic model | [runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Parity harness | Eight-prompt native/CLI/browser bit-identity on the seed-5 synthetic stack, with fresh-process restore | [M7 parity record](../labs/m7-parity-library-cli-browser/README.md) |
| Status text | Pre-alpha, 1 of 8 milestones complete; M1 decode measured separately from the synthetic stand-in; no qualified model bundled | [`index.html`](../../web/studio/index.html) |
| Public deploy | Old Studio deploy remains paused. The separate mdBook workflow publishes preview on main and production only by manual dispatch | [`deploy.yml`](../../.github/workflows/deploy.yml), [`docs-site.yml`](../../.github/workflows/docs-site.yml) |

The SIMD kernels in the WebAssembly build are fixtures, not the hot path; the engine currently uses
its scalar table kernel. These implementation limits do not negate the M1 model's native results,
but they are not a browser qualification of that model.

## What is still missing

- **A qualified model bundle for the web product.** M1's native artifacts do not by themselves prove
  browser parity or M7 distribution.
- **The accepted M4 model and its browser export.** Full quality, session and audit bars come before
  M7's accepted-model parity check.
- **A download-once cache and release loader.**
- **Bit-identical tokens and sessions on the real model across library, CLI and browser.** The
  synthetic harness passing is an instrument result, not M7 completion.
- **Production publication of the updated Studio.** No manual production `gh-pages` deployment is
  implied by this documentation refresh.

## Privacy and integrity

- **The engine computes locally.** No prompt or reply needs a model server. Today that is trivially
  true because there is no remote model to call.
- **Page-load resources are self-hosted.** The static and headless-Chrome checks report zero external
  requests on both Studio pages. User-initiated GitHub features are a separate, explicit action and
  are refused in Tauri/offline mode.
- **A checksum proves integrity, not secrecy or quality.** The model is public; a hash identifies
  the released bytes, not who made them or whether they answer well.

Until M7 and M8 pass, the page must remain a geometry explorer and synthetic-engine demonstration,
not pretend to be a working assistant. M1's language-base result is real, but it does not change
that product boundary.

## Open questions

- How large will the accepted model file be, and is a one-time download acceptable on a phone?
- How fast is WebAssembly against native on the same file? The M1 pair's native 10,876/10,566 tok/s
  does not answer that. Older FBS native rates and older evaluation throughput are not browser rates.
- Which remaining served-memory/readout parts are needed for the accepted model's full browser path?

## Go deeper

- [Desktop client](./desktop-client.md), which shares the interface.
- [Serving and export](../model/serving-and-export.md) and the [integer serving dossier](../mechanisms/lut-serving.md).
- [WebAssembly runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md).
- [D36](../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands) and [charter](../../CHARTER.md), section 3.6.
