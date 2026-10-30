# web

`web/studio/` holds the Studio: a clean editor, chat and model-source page (`index.html`, `index.css`, `assets/js/`), the wasm worker, the build/publish helpers and the verification harnesses, plus the standalone geometry explorer (`geometric_prime_router_webapp.html`). It is the repository's web client; see [`docs/products/web-client.md`](../docs/products/web-client.md) and [`web/studio/README.md`](studio/README.md) for the current state, the build recipe and the limitations.

The WASM bundle, the demo stack, the tokenizer, the page-load/worker/parity checks and the publish step are one command (`web/studio/build_serve_wasm.sh`) plus the publisher (`scripts/studio/build-and-publish.sh`, dry run by default; `--push --yes` publishes). The published site is served by [`docs-site.yml`](../.github/workflows/docs-site.yml): the book is at the root and the Studio at `/studio/`; the workflow preserves an existing `studio/` directory, so a manual publish persists across later book deploys. `.github/**` is an owner path: the workflow change that would build the Studio automatically is proposed in the unit's REPORT, not applied. The old `deploy.yml` job remains disabled (D36).

## Build the WASM bundle and verify the Studio worker locally

Build the `crates/uor-r4-integer` (`serving-wasm`) WebAssembly bundle into the gitignored `web/studio/pkg/` directory, generate the deterministic synthetic stack fixture (`synthetic-stack.lut`), and run the automated Studio worker verification and D11 instruction audit:

```sh
RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt --out-dir ../../web/studio/pkg -- --features serving-wasm
~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack -- 5 web/studio/pkg/synthetic-stack.lut
node web/studio/verify_studio_worker.mjs
node web/studio/verify_studio_page.mjs   # static page-load references; --strict-router also covers the demo page
node web/studio/verify_studio_page_load.mjs   # runtime page-load trap in headless Chrome; --strict-router also requires the demo page clean
python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap
```

Then serve `web/studio/` with any static file server and open it in a browser:

```sh
ruby -run -ehttpd web/studio -p 8000   # then open http://localhost:8000/
```
