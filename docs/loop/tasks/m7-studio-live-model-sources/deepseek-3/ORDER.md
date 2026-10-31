# ORDER: m7-studio-live-model-sources, deepseek-3 (serve the trained M1 model)

Lab: deepseek. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Base: origin/main after the /7 engine unit. Owner direction (30 Oct 2026): "you can release a trained model on huggingface and serve it also". This unit serves the trained M1 model in the Studio; the /7 turn memory itself was ordered and landed under turn-memory-integer.

## Goal

Release the M1-closing 19-table discrete trunk pair (`uor-r4.lut-discrete-trunk/1`) publicly with its stack-bound tokenizers, and serve seed 1 in the Studio: the primary Model Source card downloads it from the Hugging Face resolve URL only on an explicit click, verifies sha256 and the tokenizer binding, loads it into a new wasm discrete serving surface and switches the page to a top-8 next-token predictor. The discrete engine reports next-token prediction only; the page refuses chat generation on it honestly. The synthetic demo chat, local-file and URL loaders, and the API manager are unchanged. The worker also wires the /7 memory exports (memoryInfo/markReplyStart and no session restore) so the engine unit is reachable from the page. Keep the 86.5 MB artifact out of the published site (remote-only card) and no owner path or private artifact.

## Cancel probe

- The /7 engine unit is on main (parse builds the memory; memoryInfo/markReplyStart exports exist) and the public HF release of the M1 trunk answers anonymous 200 with CORS; the Studio page still has no trained-model card.
If either fails, stop and report.

## Files in scope

`crates/uor-r4-integer/src/{wasm_serve,wasm_bindgen_exports}.rs` (discrete exports), `web/studio/**`, `scripts/stage-studio-site.sh`, `scripts/studio/build-and-publish.sh`, `docs/products/web-client.md`, `web/README.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/m7-studio-live-model-sources.md`, `docs/loop/tasks/m7-studio-live-model-sources/deepseek-3/**`.

## Done when

- public releases answer anonymous 200 with the recorded sha256 and CORS: `caseyallard/uor-r4-m1-model` (19-table trunk pair + tokenizers), `caseyallard/uor-r4-m1-discrete` (4-table base), `caseyallard/uor-r4-fbs-dual-c131` (research trunk pair);
- wasm builds with `loadDiscrete`/`predictDiscrete`/`discreteInfo` plus the /7 memory exports; worker loads both discrete schemas and `uor-r4.lut-stack/1-7`, reports `memoryInfo`, calls `markReplyStart` before the first generated token and refuses memory-session restore/export/import; the synthetic demo, parity and page-load traps pass (external [] at load);
- a staged headless run loads the published M1 seed-1 trunk via the primary card (sha256 + tokenizer verified), switches to the predictor and returns the deterministic top-8 for the test prompt (`[1,331,454,16,90,310,455,519]`), with external requests only after the click; generate on the discrete model is refused with the next-token reason;
- publisher dry run with `--trained-manifest` stages the page without the 86.5 MB artifact and pushes nothing;
- roadmap check/render --check current, claim-wording and git diff --check pass, loop pr-check --local PASS, five required checks carry `policy: pass`.

## Stop and ask if

A test is weakened, an owner path or private artifact would move, the page would fetch an external host at load, or the published model cannot be loaded and hashed at its URL.
