# ORDER: m7-studio-connect-wasm-engine, antigravity-21

Lab: antigravity. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md). Base: `d32fc17a4`.

## Goal

Repair the four required findings and one low finding from Claude's `CHANGES` review on PR `#2978` at `d32fc17a4455d04fef5b8340a8e4a31cc01e3b44` ([`#2978` comment 6346135496](https://github.com/UOR-Foundation/uor-r4/pull/2978#issuecomment-6346135496)):

1. **HIGH (unsourced `35.1% tok/s vs 50% bar` figure):** Replace every unsourced `"35.1% tok/s"` / `"35.1% tok/s vs 50% bar"` string in `web/studio/index.html` with the sourced facts (`0/8` milestones complete; `35.1%` is the A8 preflight checkpoint-0 `P_succ` admission rate in `docs/labs/address-cells-preflight-2026-10-19/README.md`, not a `tok/s` measurement; D11 decode `tok/s` on a qualified model is unmeasured).
2. **MEDIUM (`close_session` / `reset_session` mid-generation race):** Add the `activeJob` guard to `close_session` and `reset_session` in `web/studio/assets/js/uor_model_worker.js` and add a per-conversation epoch check inside `generate` so mid-generation close or reset cannot corrupt WASM state or resurrect a closed conversation; cancel any active generation job before closing/resetting in `web/studio/index.html`; add a deterministic race regression test in `web/studio/verify_studio_worker.mjs`.
3. **MEDIUM (scope `done` vs remaining UI CDN assets):** Keep `docs/roadmap/items/m7-studio-connect-wasm-engine.md` at `status: ready` (with `References #2868` in `REPORT.md`), scope all client-side claims across `web/studio/index.html`, `web/README.md`, `docs/products/web-client.md`, `docs/labs/m7-studio-connect-wasm-engine/README.md`, and `REPORT.md` to **the model path only**, and record the remaining third-party UI CDN scripts (`Google Fonts`, `marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) in `## What did not work, first`.
4. **MEDIUM (strengthen `verify_studio_worker.mjs`):** Stub and trap `navigator.sendBeacon`, `EventSource`, `importScripts`, `Worker`, `SharedWorker`, `fetch`, `XMLHttpRequest`, and `WebSocket` in `web/studio/verify_studio_worker.mjs`, verify the traps catch injected `fetch`, `sendBeacon`, and `EventSource` payloads in a self-test, and scan `uor_model_worker.js` and `index.html` prompt pathways for any outbound network APIs.
5. **LOW (stand-in prompt tokenizer label):** Explicitly label the prompt tokenizer (`rawBytes[i] % vocabSize`) as `"UTF-8 byte-mod stand-in tokenizer"` in `uor_model_worker.js` and `web/studio/index.html`.

## Cancel probe (run first)

Verified before starting the repair:
- `uor-harness loop next` from `/Users/casey.allard/uor-r4/.worktrees/antigravity-main-verify` reports `kill-switch: off` and `GO repair #2978 — the last counted review record at its head (claude) says CHANGES`.

## Files in scope

- `web/**`
- `crates/uor-r4-integer/**`
- `docs/products/**`
- `docs/labs/m7-studio-connect-wasm-engine/**`

## Done when

1. Every unsourced `"35.1% tok/s"` / `"35.1% tok/s vs 50% bar"` reference in `web/studio/index.html` and docs is replaced with the sourced A8 preflight `P_succ = 35.1%` citation (`docs/labs/address-cells-preflight-2026-10-19/README.md:71`) and explicit statement that D11 decode `tok/s` on a qualified model is unmeasured.
2. `close_session` and `reset_session` in `web/studio/assets/js/uor_model_worker.js` enforce the `activeJob` guard and per-conversation epoch check, and `web/studio/verify_studio_worker.mjs` tests mid-generation `close_session` and `reset_session` refusal + post-cancel state integrity.
3. `docs/roadmap/items/m7-studio-connect-wasm-engine.md` remains `status: ready` (`References #2868`), all docs and `index.html` scope client-side claims to the model path only, and `REPORT.md` lists the remaining UI CDN assets in `## What did not work, first`.
4. `web/studio/verify_studio_worker.mjs` traps `navigator.sendBeacon`, `EventSource`, `importScripts`, `Worker`, `SharedWorker`, `fetch`, `XMLHttpRequest`, and `WebSocket`, self-tests the traps, and passes (`ALL STUDIO WORKER CHECKS PASSED`).
5. `web/studio/assets/js/uor_model_worker.js` and `web/studio/index.html` label the prompt tokenizer as a stand-in (`UTF-8 byte-mod stand-in`).

## Stop and ask if

- Any file outside the ORDER's scope needs modification.
