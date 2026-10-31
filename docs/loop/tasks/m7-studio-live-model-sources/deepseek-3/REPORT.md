# REPORT: m7-studio-live-model-sources, deepseek-3 (serve the trained M1 model)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first page-side load of the published M1 trunk failed: `model_sources.js` passed a `Uint8Array` to a header parser that built `new DataView(argument)`, so the browser reported "First argument to DataView constructor must be an ArrayBuffer". The parser now accepts both an ArrayBuffer and a typed-array view (byte offset/length honoured). The post-merge headless E2E caught it.
- The trained FBS trunk pair has no trained writer: `uor-chat --stack` reports "the engine returned 0 scores for a vocabulary of 4096" and the session repeats token 0/1. It is released as a research artifact only and labelled not generation-capable; the served trained model is the M1 discrete trunk.
- The M1 model is a language base, not a chat model: greedy continuation repeats the last token (measured on both seeds; a prompt ending in token 16 predicts 16). The Studio therefore serves **top-8 next-token prediction**, labels it honestly and refuses generation on the discrete engine.
- The 86.5 MB trunk is not bundled in the site: re-committing it on every publish is wrong, so the primary card downloads it from the Hugging Face resolve URL after an explicit click (CORS verified; credentials:'omit'). Page/worker cap is 128 MB; the page makes no external request at load (external []).
- A /7 memory session cannot be restored yet (the engine unit's turn state is not saved), so the worker refuses restore/export/import for it and says so. The /7 serving itself landed in the engine PR; this unit wires it into the worker and page.
- Automation of the publish remains an owner-only `.github/workflows/docs-site.yml` proposal in the deploy report; no owner path is touched here.

## What changed

- **Wasm/exports:** `crates/uor-r4-integer/src/wasm_serve.rs` and `wasm_bindgen_exports.rs` gain `loadDiscrete`/`predictDiscrete`/`discreteInfo` for the native discrete runtime (both `uor-r4.lut-discrete/1` and `-trunk/1`).
- **Worker:** loads `uor-r4.lut-stack/1-7` and both discrete schemas; discrete path returns top-8 predictions; memory path reports `memoryInfo`, marks the reply start before the first generated token and refuses memory-session restore/export/import; 128 MB cap; engine_info served /1-/7, refused [].
- **Page:** the primary Model Source card loads the published M1 seed-1 trunk from the Hugging Face URL on click, verifies sha256 and the stack-bound tokenizer, switches to the next-token predictor with decoded tokens, ms and sha/schema/size/load-time chips; generic cross-origin published entries with tokenizer_url/sha256/size pins; the synthetic demo chat, local-file and URL loaders and the API manager are unchanged.
- **Publisher:** `scripts/stage-studio-site.sh --trained-manifest` (1-4 entries, same-origin and/or HF-remote cards, 100 MB/file check) and `scripts/studio/build-and-publish.sh --trained-manifest` pass-through; the staged site stays 1.5 MB.
- **Records:** item follow-up section, `docs/products/web-client.md`, notebook M1 row, `web/studio/README.md`, this ORDER/REPORT; the item scope gains the two wasm paths.

## Measurements

- Combined wasm (engine + discrete exports): 626,889 B sha256 `d7bcaf40…`; worker verify ALL PASSED (32-token synthetic trajectory, save/restore bit-identical, remoteRequests 0, served /1-/7, refused [], turn_memory_served true); parity 8/8 `worker_matches_wasm`; static and runtime page-load traps external [].
- Headless Chrome (staged site, published HF URL; re-run at this head): page-load external requests 0; M1 seed-1 trunk 86,536,132 B downloaded, sha256 04652a3d… verified, tokenizer a9fb13f7… bound, loaded in 8.1 s (82.5 MB size chip), Predict 3 ms; deterministic top-8 `[1,331,454,16,90,310,455,519]` decoded `['<|eos|>',' with',' did','.','x',' H','gether',' me']`; generate refused with the next-token reason. This is a single local run of an uncommitted helper; the reviewer reproduced the published artifact sha and the native greedy top1=1, not the full browser ordering.
- Public releases (anonymous 200, exact bytes, CORS): `caseyallard/uor-r4-m1-model` 86,536,132 B sha256 04652a3d… + 84,943,328 B sha256 1b49e8bd… + tokenizers a9fb13f7…/f804ecbe…; `caseyallard/uor-r4-m1-discrete` 542,260 B sha256 fc31bcc7…/a75ea7f5… + tokenizer d36d3e87…; `caseyallard/uor-r4-fbs-dual-c131` 2,144,960 B sha256 a9df5263…/da56fe96… + tokenizers 1ca1ad2f…/5c09890c…; MIT.
- Publisher dry run with the trained manifest: 21 studio files / 1,556,480 B (models.json text varies with the named manifest; the site is ~1.5 MB and carries no 86.5 MB artifact), with the primary M1 card, M1 seed 2 and the two research FBS cards in models.json; nothing pushed.

## Checks run at the head

- `web/studio/build_serve_wasm.sh` -> combined wasm 626,889 B sha256 `d7bcaf40…`, worker verify PASS.
- `cargo test -p uor-r4-integer --features serving-wasm --lib` -> 372 passed / 0 failed / 1 ignored (engine tests unchanged; the /7 unit passed in its own PR).
- `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap` -> 7/7, float 0, step mul 26 / div 12.
- `node web/studio/verify_m7_parity.mjs` -> 8/8; `verify_studio_page.mjs` + `verify_studio_page_load.mjs` -> external []; `verify_studio_client_desktop...` -> `verify_desktop_client.mjs` PASS, pageWired:false.
- Headless M1 E2E above; publisher dry run; claim-wording, git diff --check, roadmap check/render --check, `loop pr-check --local` -> pass.
- Changed crates fmt-clean; the workspace `fmt --all` diffs are pre-existing files untouched here (identical on origin/main).

## Skills used

working-discipline, scope-guard, solution-integrity, uor-research-architect; reports: `local/lead/studio-model/serve-report.md`, `local/lead/model-release/{m1-release,m1-trunk-release}.md`.

## Still open

- No trained writer for the FBS trunk exports (research release only); the M1 model is a next-token language base and greedy continuation repeats tokens; browser coverage is macOS Chrome only; the 86.5 MB model needs network and ~10 s in the browser.
- Turn-memory units 4-5 (GPU DEV parity under a pre-registered sheet; cost) remain open; no trained /7 artifact exists yet; a memory session's restore is refused until its turn state is saved.
- The owner-only docs-site.yml automation patch is still a proposal; the manual publisher (now with --trained-manifest) is the path.

## Plain summary

The trained M1 model is released publicly and served in the Studio as top-8 next-token prediction from its Hugging Face release, with hash and tokenizer verification, an honest language-base-not-chat label, and no external request at page load; the /7 turn memory is wired through the worker with honest restore limits. No private artifact, owner path or committed binary moved; the remaining work is the turn-memory GPU parity/cost units and the owner-only workflow automation patch.
