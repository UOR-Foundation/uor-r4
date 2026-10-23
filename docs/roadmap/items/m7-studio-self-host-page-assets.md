---
id: m7-studio-self-host-page-assets
title: Self-host the Studio page-level UI assets and bundle the tokenizer
milestone: M7
area: serving
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [web/**, docs/products/**, docs/labs/m7-studio-self-host-page-assets/**]
---

## Why it matters

The Studio's model worker is fully client-side (m7-studio-connect-wasm-engine, PR #2978), but the page itself still loads UI fonts and scripts from external CDNs (Google Fonts, `marked`, KaTeX, pdf.js, three.js, Monaco — `web/studio/index.html` lines 11–25, 3711, 5164), and the prompt tokenizer is a UTF-8 byte-mod stand-in (`rawBytes[i] % vocabSize`). A page that phones CDNs on load is not a private local product, and a stand-in tokenizer cannot be shipped next to a real model.

## What is missing

Self-hosted copies (or removal) of every page-level external asset, with the worker's network-trap verification extended to page load so zero external requests is checked, not asserted; and a stack-bound tokenizer artifact loaded by the page in place of the byte-mod stand-in. Website deploys stay paused (D36) regardless; this item changes assets, not the deploy.

## Done when

`web/studio/index.html` makes zero external requests on load (fonts, scripts and assets self-hosted or removed), verified by extending `web/studio/verify_studio_worker.mjs`'s trap set to page load; the prompt tokenizer is the stack-bound artifact with the stand-in removed; `docs/products/web-client.md` updated to match.

## How to start

1. Read the "Still open" list on #2868 and PR #2978's report, then `web/studio/index.html` lines 11–25, 3711 and 5164 for the current external loads.
2. Post a design note on the issue listing every external asset, whether it is self-hosted or removed, and how the tokenizer artifact reaches the page.
3. Self-host or remove the assets, bundle the tokenizer artifact, extend the trap verification to page load.
4. Open a PR that says `Closes #<this issue>`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): no softmax or squashing, no dense matrix multiplication, including in any asset the page ships.
- **Records** (§4.1, §5.6): one row in `docs/research/notebook/m4-serving.md`.

## Sources

- [m7-studio-connect-wasm-engine](m7-studio-connect-wasm-engine.md) (resolution note)
- issue #2868 ("Still open" list)
- [`docs/products/web-client.md`](../../products/web-client.md)
