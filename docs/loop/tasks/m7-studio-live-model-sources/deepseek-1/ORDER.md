# ORDER: m7-studio-live-model-sources, deepseek-1 (fix the Studio; publish is the post-merge step)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: `7d79ecf23`. Owner direction, 2026-10-30 18:14Z and answers 18:25Z on #2028: fix the Studio at /studio/, make it cleaner, make the wasm real, ship the bundled demo + local-file + URL + OpenAI-compatible API sources, do not publish private HF artifacts, publish live directly after review.

## Goal

Land the Studio fix as one reviewed unit: the cleaner offline page (honest current status, model-source panel with bundled demo / local file / user URL / OpenAI-compatible API manager add-edit-remove), the real wasm bundle build (crates/uor-r4-integer serving-wasm, wasm-pack, synthetic seed-5 stack + tokenizer), the worker accepting the engine's served stack schemas and page-resolved model bytes and reporting the /7 turn-memory refusal honestly, updated non-vacuous verification harnesses, the build/publish helper scripts/studio/build-and-publish.sh + scripts/stage-studio-site.sh (dry run by default), docs update, notebook row, FILEMAP row.

The four content commits re-create the specialist branch trees unchanged as new commits with identical trees and Lab/Order trailers (studio-front 17a76dc0a; studio-wasm 6ff6c8371 and c253cdc04; studio-pages 9092b9965); this ORDER comes first and the REPORT is its pair. The .github/** workflow change (build wasm + studio in docs-site.yml) remains an owner-only proposal in the REPORT, not applied.

**The item stays ready in this PR: its Done-when 4 (live publish) is the post-merge step.** After this PR merges, the Lead runs scripts/studio/build-and-publish.sh --target gh-pages --push --yes from an attached main, verifies the live page, then a one-line records PR sets the item done, records the live URL/hashes and closes #3440 (D70).

## Cancel probe (run first)

- Live /studio/ is still the 13 October deploy (old router engine, CDN loads); docs-site.yml builds no wasm and preserves studio/; web/studio/pkg is gitignored and absent from the repo.
- The engine serves uor-r4.lut-stack/1-6 (a /7 turn-memory stack is refused) and the source page already has a /1-only local sidecar loader.
- The owner answers on #2028 (comment 6534841207) and the item #3440 exist; gh-pages head is recorded for rollback (bac74f13 at the time of writing).
If any fails, stop and report.

## Files in scope

- `web/studio/**` (page, CSS, new model_sources.js/studio_api.js, worker, tokenizer tool, build_serve_wasm.sh, verify scripts)
- `scripts/stage-studio-site.sh`, `scripts/studio/build-and-publish.sh`
- `docs/products/web-client.md`, `web/README.md`, `web/studio/README.md`
- `docs/research/notebook/m4-serving.md`, `docs/start/FILEMAP.md`
- `docs/loop/tasks/m7-studio-live-model-sources/deepseek-1/**`

## Done when

At the final head, run and paste as the review's done-when host line:
- web/studio/build_serve_wasm.sh builds pkg (wasm 495,334 B sha256 d971fe7d…, demo 469,252 B sha256 d60f88d6…, tokenizer 1,206 B) and the worker verification passes (32-token trajectory, save/restore bit-identical, allocDelta 0, remoteRequests 0);
- verify_m7_parity.mjs passes on 8 prompts; the static page scan and the headless-Chrome page-load trap pass with external [] on both pages (same-origin bundled assets only); scripts/audit_zero_matmul_wasm.py gives 7/7 with float 0;
- verify_desktop_client.mjs passes and states the desktop contract honestly (pageWired:false);
- the worker reports served schemas /1-6 and the /7 turn-memory refusal in engine_info, with a verifier assertion;
- scripts/studio/build-and-publish.sh dry run stages studio/ + models.json and pushes nothing; the live publish command is recorded for the post-merge step;
- roadmap check 115 items OK, claim-wording and git diff --check pass, uor-harness loop pr-check --local PASS, and the five required checks carry policy: pass (run <id>).

## Stop and ask if

Any generated binary would be committed; a test is weakened or skipped; the page makes an external/off-origin request at load; the live publish needs an owner path or a setting change; or the item's claims stop matching what was verified.
