# ORDER: m7-studio-connect-wasm-engine, deepseek-51 (Studio ↔ browser WASM engine)

Lab: deepseek. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) (#2868 / M7 #2035). Base: `35a373de1`. `References #2868` (Closes only when the Studio is rewired, the audit is 7/7 and a real model exists; a synthetic stand-in is labelled).

## Goal

Connect the Studio page to the live `serving-wasm` engine (`loadStack`, `step`, `saveSession`, `restoreSession`) so the model runs entirely client-side in a Web Worker, with truthful capability reporting ("synthetic stand-in" when the synthetic stack is loaded), zero remote model calls, and the `.wasm` module passing `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7, mul=0, div=0, float=0). The retired router engine path and its banner are removed. Pages deploy stays paused (D36).

## Cancel probe (run first)

Class **instrument/build** (host-only; no GPU). At the base: `grep -c 'uor_r4_wasm_router' web/studio/index.html web/studio/assets/js/uor_model_worker.js` finds the retired engine still referenced, and `crates/uor-r4-integer/src/wasm_bindgen_exports.rs` exposes the new exports. Host toolchain present (`wasm-pack`, `wasm32-unknown-unknown`, node 26.4). Task stands; if a build step needs a tool that is missing, stop and ask instead of installing project dependencies silently.

## Files in scope

- `web/**` (Studio page, worker, tests), `crates/uor-r4-integer/**` (only if an export/capability field is missing), `docs/products/**`, `docs/labs/m7-studio-connect-wasm-engine/**`
- this task folder

## Plan (one unit per session; ORDER first, REPORT before queue)

1. Read `docs/products/web-client.md`, `wasm_bindgen_exports.rs`, `wasm_serve.rs`, `web/studio/assets/js/uor_model_worker.js`, `web/studio/r4_worker.js`, and the `initModelWorker`/`d36EngineStatusBanner` sites in `web/studio/index.html`. Post a design note on #2868: page↔worker message protocol mapped onto the four exports; tokenizer path; capability fields and their source; every page network request with its removal/opt-in plan; the no-softmax/no-matmul/no-float-sampling statement.
2. Build the bundle: `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`; generate the synthetic stack with the `mk-synthetic-stack` bin; copy `pkg/` + the synthetic stack next to `docs/wasm-serving/verify.mjs` and run it under Node to prove the bundle before touching the Studio.
3. First deliverable: a failing Node/browser test that loads the synthetic stack through the Studio worker and matches `verify.mjs`'s tokens; then rewire the worker to the new exports and delete the retired router path/banner.
4. Truthful capability panel (from the engine's own returns; "synthetic stand-in" label) plus an automated check that no request carries model input (zero remote model calls).
5. Run the wasm audit and post its 7/7 output; add the notebook row to `docs/research/notebook/m4-serving.md`; `uor-harness check`, `loop report --check`, `loop pr-check --local`, `loop gates --full`.

## Done when

- The Studio worker uses the new exports client-side, the retired engine is gone, capability reporting is truthful, a Node/browser test pins token parity against `verify.mjs` on the synthetic stack, and the audit posts 7/7 (`mul=0`, `div=0`, `float=0`).
- Pages deploy untouched (D36); no owner path; `References #2868` with a counted non-author review and a verified merge.

## Stop and ask if

- A network request carrying model input cannot be removed without a product decision, the Pages deploy or an owner path must change, a tool must be installed project-wide, or a real model (not the synthetic stand-in) would be required to meet the criterion.